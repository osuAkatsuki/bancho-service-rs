mod protocol;
mod rooms;

use crate::commands::CommandTarget;
use crate::common::chat::safe_username;
use crate::common::context::PoolContext;
use crate::common::error::{AppError, ServiceResult};
use crate::common::state::AppState;
use crate::entities::bot;
use crate::models::messages::Recipient;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use crate::settings::AppSettings;
use crate::usecases::{bancho_settings, messages};
use chrono::Utc;
use md5::{Digest, Md5};
use protocol::{Client, Command};
use rooms::Rooms;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio::time::{Duration, Instant, MissedTickBehavior, interval, timeout};
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
use uuid::Uuid;

const SERVER: &str = "irc.akatsuki.gg";
const REGISTRATION_TIMEOUT: Duration = Duration::from_secs(20);
const AUTH_RECHECK: Duration = Duration::from_secs(5);
const PING_INTERVAL: Duration = Duration::from_secs(45);
const IDLE_TIMEOUT: Duration = Duration::from_secs(120);

pub async fn serve(settings: &AppSettings) -> anyhow::Result<()> {
    let certs = CertificateDer::pem_file_iter(env::var("IRC_TLS_CERT_PATH")?)?
        .collect::<Result<Vec<_>, _>>()?;
    let key = PrivateKeyDer::from_pem_file(env::var("IRC_TLS_KEY_PATH")?)?;
    let tls = TlsAcceptor::from(Arc::new(
        ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)?,
    ));
    let state = crate::lifecycle::initialize_state(settings).await?;
    let address = SocketAddr::from((settings.app_host, settings.app_port));
    let listener = TcpListener::bind(address).await?;
    let capacity = Arc::new(Semaphore::new(64));
    tracing::info!(%address, "Listening for IRC over TLS");

    loop {
        let (socket, peer) = listener.accept().await?;
        let Ok(permit) = Arc::clone(&capacity).try_acquire_owned() else {
            continue;
        };
        let tls = tls.clone();
        let state = state.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let Ok(Ok(socket)) = timeout(REGISTRATION_TIMEOUT, tls.accept(socket)).await else {
                return;
            };
            let (reader, writer) = tokio::io::split(socket);
            let mut client = Client::new(reader, writer);
            if let Err(error) = connection(&state, &mut client, peer).await {
                tracing::debug!(%peer, %error, "IRC connection ended");
            }
        });
    }
}

#[derive(Default)]
struct Registration {
    nick: Option<String>,
    token_digest: Option<String>,
    user_received: bool,
    negotiating_caps: bool,
}

impl Registration {
    fn credentials(&self) -> Option<(&str, &str)> {
        if self.negotiating_caps || !self.user_received {
            return None;
        }
        Some((self.nick.as_deref()?, self.token_digest.as_deref()?))
    }
}

fn register<'a, R: AsyncRead + Unpin + Send + 'a, W: AsyncWrite + Unpin + Send + 'a>(
    state: &'a AppState,
    client: &'a mut Client<R, W>,
    peer: SocketAddr,
) -> impl Future<Output = anyhow::Result<Option<(Session, String, String)>>> + Send + 'a {
    async move {
        let mut registration = Registration::default();
        while let Some(line) = client.read().await? {
            let Some(command) = Command::parse(&line) else {
                continue;
            };
            match command.name.as_str() {
                "CAP" => {
                    let capability = command.params.split(' ').next().unwrap_or("");
                    if capability.eq_ignore_ascii_case("LS") {
                        registration.negotiating_caps = true;
                        client.send(&format!(":{SERVER} CAP * LS :")).await?;
                    } else if capability.eq_ignore_ascii_case("END") {
                        registration.negotiating_caps = false;
                    } else if capability.eq_ignore_ascii_case("REQ") {
                        client.send(&format!(":{SERVER} CAP * NAK :")).await?;
                    }
                }
                "NICK" if valid_nick(command.argument()) => {
                    registration.nick = Some(command.argument().to_owned());
                }
                "NICK" => client.numeric("432", "*", ":Invalid nickname").await?,
                "USER" => registration.user_received = true,
                "PASS" => {
                    registration.token_digest =
                        Some(format!("{:x}", Md5::digest(command.argument())));
                }
                "PING" => {
                    client
                        .send(&format!(":{SERVER} PONG {SERVER} :{}", command.argument()))
                        .await?
                }
                "QUIT" => return Ok(None),
                _ => {
                    client
                        .numeric("451", "*", ":Register with PASS, NICK and USER")
                        .await?
                }
            }
            if let Some((nick, token_digest)) = registration.credentials() {
                if bancho_settings::in_maintenance_mode(state)
                    .await
                    .map_err(service_error)?
                {
                    client
                        .numeric("465", nick, ":Bancho is in maintenance")
                        .await?;
                    return Ok(None);
                }
                let Some(account) = authenticate(state, nick, token_digest)
                    .await
                    .map_err(service_error)?
                else {
                    client
                        .numeric("464", nick, ":Invalid IRC token or account")
                        .await?;
                    return Ok(None);
                };
                client
                    .numeric("001", nick, ":Welcome to Akatsuki tournament IRC")
                    .await?;
                client
                    .numeric("005", nick, "CHANTYPES=# :are supported by this server")
                    .await?;
                client.numeric("422", nick, ":MOTD file is missing").await?;
                return Ok(Some((
                    account.into_session(peer),
                    nick.to_owned(),
                    token_digest.to_owned(),
                )));
            }
        }
        Ok(None)
    }
}

#[derive(sqlx::FromRow)]
struct IrcAccount {
    id: i32,
    username: String,
    username_safe: String,
    privileges: i32,
    silence_end: i64,
}

impl IrcAccount {
    fn privileges(&self) -> Privileges {
        Privileges::from_bits_retain(self.privileges)
    }

    fn silence_end(&self) -> Option<chrono::DateTime<Utc>> {
        chrono::DateTime::from_timestamp(self.silence_end, 0)
    }

    fn into_session(self, peer: SocketAddr) -> Session {
        let privileges = self.privileges();
        let silence_end = self.silence_end();
        Session {
            session_id: Uuid::new_v4(),
            user_id: self.id.into(),
            username: self.username,
            privileges,
            create_ip_address: peer.ip(),
            private_dms: true,
            silence_end,
            updated_at: Utc::now(),
        }
    }
}

async fn authenticate(
    state: &AppState,
    nick: &str,
    token_digest: &str,
) -> ServiceResult<Option<IrcAccount>> {
    let account: Option<IrcAccount> = sqlx::query_as(
        "SELECT users.id, users.username, users.username_safe, users.privileges, users.silence_end
         FROM irc_tokens JOIN users ON users.id = irc_tokens.userid
         WHERE irc_tokens.token = ?",
    )
    .bind(token_digest.to_owned())
    .fetch_optional(state.db())
    .await?;
    Ok(account.filter(|account| {
        safe_username(nick) == account.username_safe
            && account
                .privileges()
                .contains(Privileges::CanLogin | Privileges::PubliclyVisible)
            && !account.privileges().is_pending_verification()
            && i64::from(account.id) != bot::BOT_ID
    }))
}

fn refresh<'a>(
    state: &'a AppState,
    session: &'a mut Session,
    nick: &'a str,
    token_digest: &'a str,
) -> impl Future<Output = ServiceResult<bool>> + Send + 'a {
    async move {
        if bancho_settings::in_maintenance_mode(state).await? {
            return Ok(false);
        }
        match authenticate(state, nick, token_digest).await? {
            Some(current) if session.user_id == i64::from(current.id) => {
                session.privileges = current.privileges();
                session.silence_end = current.silence_end();
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

async fn connection<R: AsyncRead + Unpin + Send, W: AsyncWrite + Unpin + Send>(
    state: &AppState,
    client: &mut Client<R, W>,
    peer: SocketAddr,
) -> anyhow::Result<()> {
    let Some((mut session, nick, token_digest)) =
        timeout(REGISTRATION_TIMEOUT, register(state, client, peer)).await??
    else {
        return Ok(());
    };
    let mut rooms = Rooms::default();
    let mut tick = interval(Duration::from_millis(500));
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut last_check = Instant::now();
    let mut last_pong = Instant::now();
    let mut last_ping = Instant::now();

    loop {
        tokio::select! {
            line = client.read() => {
                let Some(line) = line? else { return Ok(()); };
                let Some(command) = Command::parse(&line) else { continue; };
                match command.name.as_str() {
                    "QUIT" => return Ok(()),
                    "PING" => client.send(&format!(":{SERVER} PONG {SERVER} :{}", command.argument())).await?,
                    "PONG" if command.params.split_ascii_whitespace().last().is_some_and(|token| token.trim_start_matches(':') == SERVER) => last_pong = Instant::now(),
                    "JOIN" | "PRIVMSG" => {
                        if !refresh(state, &mut session, &nick, &token_digest).await.map_err(service_error)? {
                            client.send("ERROR :IRC account or token is no longer available").await?;
                            return Ok(());
                        }
                        last_check = Instant::now();
                        if command.name == "JOIN" {
                            rooms.join(state, client, &session, &nick, command.argument()).await?;
                        } else if let Some((target, message)) = command.target_and_message() {
                            if is_bot(target) {
                                bot_message(state, client, &mut session, &nick, message).await?;
                            } else {
                                rooms.message(state, client, &mut session, &nick, target, message).await?;
                            }
                        }
                    }
                    "PART" => {
                        let channel = command.params.split(' ').next().unwrap_or("");
                        rooms.part(client, &nick, channel).await?;
                    }
                    "NAMES" => client.numeric("366", &nick, ":End of NAMES list").await?,
                    "WHO" => client.numeric("315", &nick, ":End of WHO list").await?,
                    "MODE" => (),
                    "NICK" => client.numeric("502", &nick, ":Use your Akatsuki username").await?,
                    _ => client.numeric("421", &nick, ":Unknown command").await?,
                }
            }
            _ = tick.tick() => {
                if last_pong.elapsed() >= IDLE_TIMEOUT {
                    return Ok(());
                }
                if last_ping.elapsed() >= PING_INTERVAL {
                    client.send(&format!("PING :{SERVER}")).await?;
                    last_ping = Instant::now();
                }
                if last_check.elapsed() >= AUTH_RECHECK {
                    if !refresh(state, &mut session, &nick, &token_digest).await.map_err(service_error)? {
                        client.send("ERROR :IRC account or token is no longer available").await?;
                        return Ok(());
                    }
                    last_check = Instant::now();
                }
                rooms.relay(state, client, &session, &nick).await?;
            }
        }
    }
}

fn is_bot(name: &str) -> bool {
    name.eq_ignore_ascii_case(bot::BOT_NAME) || name.eq_ignore_ascii_case("BanchoBot")
}

async fn bot_message<R: AsyncRead + Unpin + Send, W: AsyncWrite + Unpin + Send>(
    state: &AppState,
    client: &mut Client<R, W>,
    session: &mut Session,
    nick: &str,
    message: &str,
) -> anyhow::Result<()> {
    if !matches!(message.split_whitespace().next(), Some("!mp")) {
        return client
            .message(bot::BOT_NAME, nick, "Use !mp make <name> to create a room.")
            .await;
    }
    match messages::send(
        state,
        session,
        &Recipient::Bot,
        message,
        CommandTarget::IrcBot,
    )
    .await
    {
        Ok(result) => {
            if let Some(answer) = result.response.and_then(|response| response.answer) {
                client.message(bot::BOT_NAME, nick, &answer).await?;
            }
        }
        Err(error) => client.message(bot::BOT_NAME, nick, error.message()).await?,
    }
    Ok(())
}

fn valid_nick(nick: &str) -> bool {
    (2..=32).contains(&nick.len())
        && nick
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-[]".contains(&byte))
}

fn service_error(error: AppError) -> anyhow::Error {
    anyhow::anyhow!("{}", error.code())
}
