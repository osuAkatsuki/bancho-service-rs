use crate::common::error::{AppError, ServiceResult};
use chrono::{DateTime, Utc};
use std::str::FromStr;

pub struct NoArg;
pub trait FromCommandArgs<'a>: Sized + Send + Sync {
    fn from_args(args: Option<&'a str>) -> ServiceResult<Self>;
    const TYPE_SIGNATURE: &'static str;
    const SYNTAX: &'static str = "args";
    const TYPED_SYNTAX: &'static str = "args";
}

impl FromCommandArgs<'_> for NoArg {
    fn from_args(_args: Option<&'_ str>) -> ServiceResult<Self> {
        Ok(NoArg)
    }
    const TYPE_SIGNATURE: &'static str = "nothing";
}

impl FromCommandArgs<'_> for String {
    fn from_args(args: Option<&'_ str>) -> ServiceResult<Self> {
        args.map(str::to_string)
            .ok_or(AppError::CommandsInvalidSyntax(
                Self::SYNTAX,
                Self::TYPE_SIGNATURE,
                Self::TYPED_SYNTAX,
            ))
    }
    const TYPE_SIGNATURE: &'static str = "string";
    const SYNTAX: &'static str = "args";
    const TYPED_SYNTAX: &'static str = const_str::concat!(
        <String as FromCommandArgs>::SYNTAX,
        ": ",
        <String as FromCommandArgs>::TYPE_SIGNATURE
    );
}

macro_rules! impl_from_args {
    ($t:ty) => {
        impl FromCommandArgs<'_> for $t {
            fn from_args(args: Option<&'_ str>) -> ServiceResult<Self> {
                match args {
                    None => Err(AppError::CommandsInvalidSyntax(Self::SYNTAX, Self::TYPE_SIGNATURE, Self::TYPED_SYNTAX)),
                    Some(args) => {
                        <$t>::from_str(args)
                            .map_err(|_| AppError::CommandsInvalidSyntax(Self::SYNTAX, Self::TYPE_SIGNATURE, Self::TYPED_SYNTAX))
                    }
                }
            }

            // TODO: use std::any::type_name::<$t>() when its stabilized
            const TYPE_SIGNATURE: &'static str = stringify!($t);
            const SYNTAX: &'static str = "args";
            const TYPED_SYNTAX: &'static str = const_str::concat!(<$t as FromCommandArgs>::SYNTAX, ": ", <$t as FromCommandArgs>::TYPE_SIGNATURE);
        }
    };
    ($t:ty, $($ts:ty),+) => {
        impl_from_args!($t);
        impl_from_args!($($ts),+);
    }
}

impl_from_args!(
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    f32,
    f64,
    DateTime<Utc>
);

impl<'a, T: FromCommandArgs<'a>> FromCommandArgs<'a> for Option<T> {
    fn from_args(args: Option<&'a str>) -> ServiceResult<Self> {
        match args {
            Some(args) => Ok(Some(T::from_args(Some(args))?)),
            None => Ok(None),
        }
    }

    const TYPE_SIGNATURE: &'static str = "optional<>";
    const TYPED_SYNTAX: &'static str = const_str::concat!("args: optional");
}
