use crate::commands::from_args::FromCommandArgs;
use crate::commands::{CommandResponse, CommandResult};
use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult};
use crate::common::redis_pool::RedisPool;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use async_trait::async_trait;
use hashbrown::HashMap;
use sqlx::{MySql, Pool};

#[derive(Debug)]
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
    pub commands: HashMap<&'static str, RegisteredCommand>,
}

#[async_trait]
pub trait CommandHandlerProxy: Send + Sync {
    async fn handle(
        &self,
        ctx: &dyn Context,
        session: &Session,
        args: Option<&str>,
    ) -> ServiceResult<Option<CommandResponse>>;
}

struct CommandContext {
    db: Pool<MySql>,
    redis: RedisPool,
}

impl CommandContext {
    pub fn from_ctx(ctx: &dyn Context) -> Self {
        Self {
            db: ctx.db_pool().clone(),
            redis: ctx.redis_pool().clone(),
        }
    }
}

impl Context for CommandContext {
    fn db_pool(&self) -> &Pool<MySql> {
        &self.db
    }

    fn redis_pool(&self) -> &RedisPool {
        &self.redis
    }
}

impl Default for CommandProperties {
    fn default() -> Self {
        CommandProperties {
            name: "",
            forward_message: true,
            required_privileges: None,
            read_privileges: None,
        }
    }
}

#[async_trait]
impl<CMD: Command> CommandHandlerProxy for CMD {
    async fn handle(
        &self,
        ctx: &dyn Context,
        session: &Session,
        args: Option<&str>,
    ) -> ServiceResult<Option<CommandResponse>> {
        let ctx = CommandContext::from_ctx(ctx);
        let args = CMD::Args::from_args(args)?;
        let answer = CMD::handle(&ctx, session, args).await?;
        Ok(Some(CommandResponse {
            answer,
            properties: CMD::PROPERTIES,
        }))
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
            fn into_pair<B: 'static + Command>(c: B) -> (&'static str, RegisteredCommand) {
                (B::PROPERTIES.name, RegisteredCommand::new(c))
            }

            #[allow(unused_mut)]
            let mut router = CommandRouter::from([ $(into_pair($c)),* ]);
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
    ) -> ServiceResult<Option<CommandResponse>> {
        match args {
            Some(args) => {
                let mut parts = args.splitn(2, ' ');
                let cmd_name = match parts.next() {
                    Some(cmd_name) => cmd_name,
                    // No command, ignore
                    None => return Ok(None),
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
            None => Ok(None),
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
