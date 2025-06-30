use crate::commands::from_args::FromCommandArgs;
use crate::commands::{CommandResponse, CommandResult};
use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult};
use crate::common::redis_pool::PoolResult;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use async_trait::async_trait;
use hashbrown::HashMap;
use sqlx::{MySql, Pool};

pub struct CommandProperties {
    pub name: &'static str,
    pub forward_message: bool,
    pub required_privileges: Option<Privileges>,
    pub read_privileges: Option<Privileges>,
}

#[async_trait]
pub trait Command: Send + Sync {
    type Args: FromCommandArgs;
    const PROPERTIES: CommandProperties;
    async fn handle<C: Context>(ctx: &C, session: &Session, args: Self::Args) -> CommandResult;
}

pub struct RegisteredCommand {
    pub properties: CommandProperties,
    handler: Box<dyn CommandHandlerProxy>,
}

pub type CommandRouterFactory = fn() -> CommandRouter;
pub type CommandRouterInstance = std::sync::LazyLock<CommandRouter>;
pub struct CommandRouter {
    commands: HashMap<&'static str, RegisteredCommand>,
}

#[async_trait]
pub trait CommandHandlerProxy: Send + Sync {
    async fn handle(
        &self,
        ctx: &dyn Context,
        session: &Session,
        args: Option<&str>,
    ) -> ServiceResult<CommandResponse>;
}

struct CommandContext<'a>(&'a dyn Context);
#[async_trait]
impl Context for CommandContext<'_> {
    fn db(&self) -> &Pool<MySql> {
        self.0.db()
    }
    async fn redis(&self) -> PoolResult {
        self.0.redis().await
    }
}

#[async_trait]
impl<C: Command> CommandHandlerProxy for C {
    async fn handle(
        &self,
        ctx: &dyn Context,
        session: &Session,
        args: Option<&str>,
    ) -> ServiceResult<CommandResponse> {
        let ctx = CommandContext(ctx);
        let args = C::Args::from_args(args)?;
        let response = C::handle(&ctx, session, args).await?;
        Ok(CommandResponse {
            answer: Some(response),
            properties: C::PROPERTIES,
        })
    }
}

#[macro_export]
macro_rules! commands {
    (
        include = [$( $p:expr => $cc:expr ),* $(,)?],
        $($c:path),* $(,)?
    ) => {
        || {
            use $crate::commands::{Command, CommandRouter, RegisteredCommand};
            fn _a<B: 'static + Command>(c: B) -> (&'static str, RegisteredCommand) {
                (B::PROPERTIES.name, RegisteredCommand::new(c))
            }

            #[allow(unused_mut)]
            let mut router = CommandRouter::from([ $(_a($c)),* ]);
            $(router.nest($p, $cc);)*
            router
        }
    };
    ($($c:path),* $(,)?) => {
        $crate::commands!(include=[], $($c),*)
    }
}

impl<const N: usize> From<[(&'static str, RegisteredCommand); N]> for CommandRouter {
    fn from(value: [(&'static str, RegisteredCommand); N]) -> Self {
        CommandRouter {
            commands: HashMap::from(value),
        }
    }
}

impl CommandRouter {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            commands: HashMap::with_capacity(capacity),
        }
    }

    pub fn get(&self, cmd_name: &str) -> Option<&RegisteredCommand> {
        self.commands.get(cmd_name)
    }

    pub fn register<C: 'static + Command>(&mut self, cmd: C) {
        self.commands
            .insert(C::PROPERTIES.name, RegisteredCommand::new(cmd));
    }

    pub fn nest(&mut self, name: &'static str, router: CommandRouterFactory) {
        self.commands
            .insert(name, RegisteredCommand::group(name, router()));
    }
}

#[async_trait]
impl CommandHandlerProxy for CommandRouter {
    async fn handle(
        &self,
        ctx: &dyn Context,
        session: &Session,
        args: Option<&str>,
    ) -> ServiceResult<CommandResponse> {
        match args {
            Some(args) => {
                let mut parts = args.splitn(2, ' ');
                let cmd_name = match parts.next() {
                    Some(cmd_name) => cmd_name,
                    // No command, ignore
                    None => return Err(AppError::CommandsUnknownCommand),
                };
                let args = parts.next();
                match self.get(cmd_name) {
                    Some(command) => {
                        if let Some(required_privileges) = command.properties.required_privileges
                            && !session.has_all_privileges(required_privileges)
                        {
                            return Err(AppError::CommandsUnauthorized);
                        }
                        command.handler.handle(ctx, &session, args).await
                    }
                    None => Err(AppError::CommandsUnknownCommand),
                }
            }
            None => Err(AppError::CommandsUnknownCommand),
        }
    }
}

impl RegisteredCommand {
    pub fn new<C: 'static + Command>(cmd: C) -> Self {
        Self {
            properties: C::PROPERTIES,
            handler: Box::new(cmd),
        }
    }

    pub fn group<C: 'static + CommandHandlerProxy>(name: &'static str, cmd: C) -> Self {
        Self {
            properties: CommandProperties {
                name,
                forward_message: true,
                required_privileges: None,
                read_privileges: None,
            },
            handler: Box::new(cmd),
        }
    }
}
