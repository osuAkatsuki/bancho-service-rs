use crate::common::error::{AppError, ServiceResult};
use chrono::{DateTime, Utc};
use std::str::FromStr;
use std::time::Duration;

pub struct NoArg;
pub trait FromCommandArgs: Sized {
    fn from_args(args: Option<&str>) -> ServiceResult<Self>;
    const TYPE_SIGNATURE: &'static str;
    const SYNTAX: &'static str = "args";
    const TYPED_SYNTAX: &'static str = "args";
}

impl FromCommandArgs for NoArg {
    fn from_args(_args: Option<&'_ str>) -> ServiceResult<Self> {
        Ok(NoArg)
    }
    const TYPE_SIGNATURE: &'static str = "nothing";
}

impl FromCommandArgs for String {
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
        impl FromCommandArgs for $t {
            fn from_args(args: Option<&str>) -> ServiceResult<Self> {
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

impl<T: FromCommandArgs> FromCommandArgs for Option<T> {
    fn from_args(args: Option<&str>) -> ServiceResult<Self> {
        match args {
            Some(args) => Ok(Some(T::from_args(Some(args))?)),
            None => Ok(None),
        }
    }

    const TYPE_SIGNATURE: &'static str = "optional<>";
    const TYPED_SYNTAX: &'static str = const_str::concat!("args: optional");
}

const ORDER_OF_UNITS: [(&str, u64); 5] = [
    ("w", 604800),
    ("d", 86400),
    ("h", 3600),
    ("m", 60),
    ("s", 1),
];

const INVALID_DURATION: &str = "Invalid Duration! Example: 1h30m (Possible units: w, d, h, m, s)";
fn parse_single_time(value: &str) -> ServiceResult<Duration> {
    for (unit, secs_multiplier) in ORDER_OF_UNITS {
        match value.strip_suffix(unit) {
            Some(duration) => {
                let duration = u64::from_str(duration)
                    .map_err(|_| AppError::CommandsInvalidArgument(INVALID_DURATION))?;
                return Ok(Duration::from_secs(duration * secs_multiplier));
            }
            None => {}
        }
    }

    Err(AppError::CommandsInvalidArgument(INVALID_DURATION))
}

fn parse_duration(value: &str) -> ServiceResult<Duration> {
    if value.chars().any(|c| !c.is_ascii_alphanumeric()) {
        return Err(AppError::CommandsInvalidArgument(INVALID_DURATION));
    }

    let split = value.split_inclusive(char::is_alphabetic);
    split.map(parse_single_time).sum()
}

impl FromCommandArgs for Duration {
    fn from_args(args: Option<&str>) -> ServiceResult<Self> {
        match args {
            Some(args) => parse_duration(args),
            None => Err(AppError::CommandsInvalidSyntax(
                Self::SYNTAX,
                Self::TYPE_SIGNATURE,
                Self::TYPED_SYNTAX,
            )),
        }
    }

    const TYPE_SIGNATURE: &'static str = "duration";
    const SYNTAX: &'static str = "args: duration";
    const TYPED_SYNTAX: &'static str = const_str::concat!(
        <Duration as FromCommandArgs>::SYNTAX,
        ": ",
        <Duration as FromCommandArgs>::TYPE_SIGNATURE
    );
}
