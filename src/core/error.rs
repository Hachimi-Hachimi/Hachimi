use std::fmt;

#[derive(Debug)]
pub enum Error {
    SymbolNotFound(String, String),
    HookingError(dobby_rs::DobbyHookError),
    AssemblyNotFound(String),
    ClassNotFound(String, String),
    MethodNotFound(String),
    IoError(std::io::Error),
    JsonParseError(serde_json::Error),
    GuiRendererInitError(String),
    HttpError(ureq::Error),
    PluralParsing,
    OutOfDiskSpace,
    FileHashMismatch(String)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SymbolNotFound(module_name, symbol_name) => {
                write!(f, "Symbol not found: {} (module {})", symbol_name, module_name)
            }
            Error::HookingError(e) => {
                write!(f, "Hooking failed: {}", e)
            }
            Error::AssemblyNotFound(name) => {
                write!(f, "Assembly not found: {}", name)
            },
            Error::ClassNotFound(namespace, class_name) => {
                write!(f, "Class not found: {}::{}", namespace, class_name)
            },
            Error::MethodNotFound(name) => {
                write!(f, "Method not found: {}", name)
            }
            Error::IoError(error) => {
                write!(f, "I/O error: {}", error)
            }
            Error::JsonParseError(error) => {
                write!(f, "Failed to parse JSON: {}", error)
            }
            Error::GuiRendererInitError(error) => {
                write!(f, "Failed to init GUI renderer: {}", error)
            }
            Error::PluralParsing => {
                write!(f, "Failed to parse plural expression")
            }
            Error::HttpError(error) => {
                write!(f, "HTTP error: {}", error)
            }
            Error::OutOfDiskSpace => {
                write!(f, "The system has ran out of disk space")
            }
            Error::FileHashMismatch(name) => {
                write!(f, "File hash mismatch: {}", name)
            }
        }
    }
}

impl From<dobby_rs::DobbyHookError> for Error {
    fn from(e: dobby_rs::DobbyHookError) -> Self {
        Error::HookingError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::JsonParseError(e)
    }
}

impl From<ureq::Error> for Error {
    fn from(e: ureq::Error) -> Self {
        Error::HttpError(e)
    }
}