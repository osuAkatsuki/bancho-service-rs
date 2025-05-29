use crate::commands::CommandResult;
use crate::commands::from_args::FromCommandArgs;
use crate::common::context::Context;
use crate::models::privileges::Privileges;
use crate::models::sessions::Session;
use async_trait::async_trait;
use hashbrown::HashMap;
use std::marker::PhantomData;
use std::sync::LazyLock;

pub type CommandRouterInstance = LazyLock<CommandRouter>;
pub struct RegisteredCommand {
    pub forward_message: bool,
    pub required_privileges: Option<Privileges>,
    pub read_privileges: Option<Privileges>,
    handler: Box<dyn CommandHandlerProxy>,
}

pub struct CommandRouter {
    commands: HashMap<String, RegisteredCommand>,
}

#[async_trait]
pub trait Command<Args: for<'a> FromCommandArgs<'a>>: 'static + Send + Sync {
    const NAME: &'static str;
    const FORWARD_MESSAGE: bool = true;
    const REQUIRED_PRIVILEGES: Option<Privileges> = None;
    const READ_PRIVILEGES: Option<Privileges> = None;
    async fn handle<C: Context + ?Sized>(ctx: &C, session: &Session, args: Args) -> CommandResult;
}

#[macro_export]
macro_rules! commands {
    ($($h:path),* $(,)?) => {
        std::sync::LazyLock::new(|| {
            use $crate::commands::command_handler::CommandRouter;
            const CMDS: &[&str] = &[$(stringify!($h)),*];
            #[allow(unused_mut)]
            let mut router = CommandRouter::with_capacity(CMDS.len());
            $(router.register($h);),*
            router
        })
    };
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

    pub fn register<Args: 'static + for<'a> FromCommandArgs<'a>, C: Command<Args>>(
        &mut self,
        cmd: C,
    ) {
        self.commands
            .insert(C::NAME.to_owned(), RegisteredCommand::new(cmd));
    }
}

impl RegisteredCommand {
    pub fn new<Args: 'static + for<'a> FromCommandArgs<'a>, C: Command<Args>>(cmd: C) -> Self {
        Self {
            forward_message: C::FORWARD_MESSAGE,
            required_privileges: C::REQUIRED_PRIVILEGES,
            read_privileges: C::READ_PRIVILEGES,
            handler: Box::new(Proxy::new(cmd)),
        }
    }

    pub fn handler(&self) -> Handler<'_> {
        Handler(self.handler.as_ref())
    }
}

pub struct Handler<'a>(&'a dyn CommandHandlerProxy);
impl<'a> Handler<'a> {
    pub async fn handle<C: Context>(
        &self,
        ctx: &C,
        session: &Session,
        args: Option<&'a str>,
    ) -> CommandResult {
        self.0.handle(ctx, session, args).await
    }
}

#[async_trait]
trait CommandHandlerProxy: Send + Sync {
    async fn handle<'a>(
        &self,
        ctx: &dyn Context,
        session: &Session,
        args: Option<&'a str>,
    ) -> CommandResult;
}

#[async_trait]
impl<Args: for<'a> FromCommandArgs<'a>, T: Command<Args>> CommandHandlerProxy for Proxy<Args, T> {
    async fn handle<'a>(
        &self,
        ctx: &dyn Context,
        session: &Session,
        args: Option<&'a str>,
    ) -> CommandResult {
        let args = Args::from_args(args)?;
        T::handle(ctx, session, args).await
    }
}

struct Proxy<Args: for<'a> FromCommandArgs<'a>, H: Command<Args>>(H, PhantomData<Args>);

impl<Args: for<'a> FromCommandArgs<'a>, H: Command<Args>> Proxy<Args, H> {
    pub fn new(handler: H) -> Self {
        Self(handler, PhantomData)
    }
}
