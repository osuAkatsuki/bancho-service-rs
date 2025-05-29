use crate::common::error::{AppError, ServiceResult};

pub struct NoArg;
pub trait FromCommandArgs<'a>: Sized + Send + Sync {
    fn from_args(args: Option<&'a str>) -> ServiceResult<Self>;
}

impl FromCommandArgs<'_> for NoArg {
    fn from_args(_args: Option<&'_ str>) -> ServiceResult<Self> {
        Ok(NoArg)
    }
}

impl<'a> FromCommandArgs<'a> for &'a str {
    fn from_args(args: Option<&'a str>) -> ServiceResult<Self> {
        args.ok_or(AppError::CommandsMissingArgument("args"))
    }
}

impl FromCommandArgs<'_> for String {
    fn from_args(args: Option<&'_ str>) -> ServiceResult<Self> {
        args.map(str::to_string)
            .ok_or(AppError::CommandsMissingArgument("args"))
    }
}

impl<'a, T: FromCommandArgs<'a>> FromCommandArgs<'a> for Option<T> {
    fn from_args(args: Option<&'a str>) -> ServiceResult<Self> {
        match args {
            Some(args) => Ok(Some(T::from_args(Some(args))?)),
            None => Ok(None),
        }
    }
}
