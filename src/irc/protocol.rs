use futures_util::StreamExt;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::time::{Duration, timeout};
use tokio_util::codec::{FramedRead, LinesCodec};

use super::SERVER;

const MAX_LINE_BYTES: usize = 510;

pub struct Client<R, W> {
    reader: FramedRead<R, LinesCodec>,
    writer: W,
}

impl<R: AsyncRead + Unpin + Send, W: AsyncWrite + Unpin + Send> Client<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader: FramedRead::new(reader, LinesCodec::new_with_max_length(MAX_LINE_BYTES + 1)),
            writer,
        }
    }

    pub async fn read(&mut self) -> anyhow::Result<Option<String>> {
        let line = self.reader.next().await.transpose()?;
        if line
            .as_ref()
            .is_some_and(|line| line.len() > MAX_LINE_BYTES || line.contains(['\0', '\r']))
        {
            anyhow::bail!("Invalid IRC line");
        }
        Ok(line)
    }

    pub fn send<'a>(
        &'a mut self,
        line: &str,
    ) -> impl Future<Output = anyhow::Result<()>> + Send + 'a + use<'a, R, W> {
        let mut end = line.len().min(MAX_LINE_BYTES);
        while !line.is_char_boundary(end) {
            end -= 1;
        }
        let mut line = line[..end].to_owned();
        async move {
            anyhow::ensure!(!line.contains(['\0', '\r', '\n']), "Invalid IRC output");
            line.push_str("\r\n");
            timeout(
                Duration::from_secs(5),
                self.writer.write_all(line.as_bytes()),
            )
            .await??;
            Ok(())
        }
    }

    pub fn numeric<'a>(
        &'a mut self,
        code: &str,
        nick: &str,
        message: &str,
    ) -> impl Future<Output = anyhow::Result<()>> + Send + 'a + use<'a, R, W> {
        self.send(&format!(":{SERVER} {code} {nick} {message}"))
    }

    pub fn message<'a>(
        &'a mut self,
        sender: &str,
        target: &str,
        text: &str,
    ) -> impl Future<Output = anyhow::Result<()>> + Send + 'a + use<'a, R, W> {
        let lines: Vec<_> = text
            .lines()
            .map(|line| {
                let line = line.replace(['\0', '\r'], "");
                format!(":{sender}!{sender}@{SERVER} PRIVMSG {target} :{line}")
            })
            .collect();
        async move {
            for line in lines {
                self.send(&line).await?;
            }
            Ok(())
        }
    }
}

pub struct Command<'a> {
    pub name: String,
    pub params: &'a str,
}

impl<'a> Command<'a> {
    pub fn parse(line: &'a str) -> Option<Self> {
        let line = line.trim_start_matches(' ');
        if line.is_empty() || line.starts_with([':', '@']) {
            return None;
        }
        let (name, params) = line.split_once(' ').unwrap_or((line, ""));
        Some(Self {
            name: name.to_ascii_uppercase(),
            params: params.trim_start_matches(' '),
        })
    }

    pub fn argument(&self) -> &'a str {
        self.params.strip_prefix(':').unwrap_or(self.params)
    }

    pub fn target_and_message(&self) -> Option<(&'a str, &'a str)> {
        let (target, message) = self.params.split_once(' ')?;
        let message = message.trim_start_matches(' ');
        let message = message.strip_prefix(':').unwrap_or(message);
        Some((target, message))
    }
}

pub fn match_id(channel: &str) -> Option<i64> {
    let id: i64 = channel.strip_prefix("#mp_")?.parse().ok()?;
    (id > 0 && channel == format!("#mp_{id}")).then_some(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_one_canonical_channel_per_match() {
        assert_eq!(match_id("#mp_65537"), Some(65537));
        for invalid in ["#mp_01", "#mp_-1", "#mp_0", "#mp_1,#mp_2", "#mp_+1"] {
            assert_eq!(match_id(invalid), None);
        }
    }

    #[test]
    fn preserves_chat_text_and_refuses_client_prefixes() {
        let command = Command::parse("privmsg #mp_123 :!mp map 1234").unwrap();
        assert_eq!(command.name, "PRIVMSG");
        assert_eq!(
            command.target_and_message(),
            Some(("#mp_123", "!mp map 1234"))
        );
        assert!(Command::parse(":Aika PRIVMSG #mp_123 :hello").is_none());
    }
}
