use crate::commands::CommandResult;
use crate::commands::from_args::FromCommandArgs;
use crate::common::context::Context;
use crate::models::sessions::Session;
use async_trait::async_trait;
use hashbrown::HashMap;
use std::marker::PhantomData;
use std::sync::LazyLock;

pub type CommandRouterInstance = LazyLock<CommandRouter>;
pub struct CommandRouter {
    commands: HashMap<String, Box<dyn CommandHandlerProxy>>,
}

#[async_trait]
pub trait Command<Args: for<'a> FromCommandArgs<'a>>: 'static + Send + Sync {
    const NAME: &'static str;
    async fn handle<C: Context + ?Sized>(ctx: &C, session: &Session, args: Args) -> CommandResult;
}

#[macro_export]
macro_rules! commands {
    ($($h:path),* $(,)?) => {
        std::sync::LazyLock::new(|| {
            use $crate::commands::command_handler::CommandRouter;
            #[allow(unused_mut)]
            let mut router = CommandRouter::new();
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

    pub fn get_handler(&self, cmd_name: &str) -> Option<Handler<'_>> {
        match self.commands.get(cmd_name) {
            Some(cmd) => Some(Handler(cmd)),
            None => None,
        }
    }

    pub fn register<Args: 'static + for<'a> FromCommandArgs<'a>, C: Command<Args>>(
        &mut self,
        cmd: C,
    ) {
        self.commands
            .insert(C::NAME.to_owned(), Box::new(Proxy::new(cmd)));
    }
}

pub struct Handler<'a>(&'a Box<dyn CommandHandlerProxy>);
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
