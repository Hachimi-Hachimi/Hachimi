use std::{fmt::Display, sync::Arc};

use super::{types::Il2CppTypeEnum, wrapper::Exception};

#[derive(Debug)]
pub enum Error {
    InvalidArgument {
        pos: u8,
        reason: Option<String>
    },
    InvalidArgumentCount {
        expected: u8,
        got: u8
    },
    Exception(Exception),
    InvokeUnboundInstanceMethod,
    AccessUnboundInstanceField,
    UnknownType(Il2CppTypeEnum),
    CoreError(crate::core::Error)
}

impl Error {
    pub fn invalid_argument(pos: u8, reason: impl Into<Option<String>>) -> Self {
        Self::InvalidArgument { pos, reason: reason.into() }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidArgument { pos, reason } =>
                write!(f, "Invalid argument #{pos}{}", reason.as_ref().map(|r| format!(": {r}")).as_deref().unwrap_or("")),

            Error::InvalidArgumentCount { expected, got } =>
                write!(f, "Invalid argument count (expected {expected}, got {got})"),

            Error::Exception(exception) =>
                f.write_str(&exception.to_string()),

            Error::InvokeUnboundInstanceMethod =>
                f.write_str("Attempted to invoke an instance method without binding an instance"),

            Error::AccessUnboundInstanceField =>
                f.write_str("Attempted to access an instance field without binding an instance"),

            Error::UnknownType(type_enum) =>
                write!(f, "Unknown type: {type_enum}"),

            Error::CoreError(e) =>
                e.fmt(f)
        }
    }
}

impl std::error::Error for Error {
}

impl From<Error> for mlua::Error {
    fn from(value: Error) -> Self {
        mlua::Error::ExternalError(Arc::new(value))
    }
}

impl From<crate::core::Error> for Error {
    fn from(value: crate::core::Error) -> Self {
        Error::CoreError(value)
    }
}