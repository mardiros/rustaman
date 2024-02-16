//! Define results and error. `Result<T, CabotError>`
use std::error::Error;
use std::fmt::{self, Display};
use std::io;

#[derive(Debug)]
pub enum RustamanError {
    RenderError(handlebars::RenderError),
    RequestParsingError(String),
    EnvironmentParsingError(serde_yaml::Error),
    UrlParseError(url::ParseError),
    IOError(io::Error),
}

/// Result used by method that can failed.
pub type RustamanResult<T> = Result<T, RustamanError>;

impl Display for RustamanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RustamanError::EnvironmentParsingError(err) => {
                write!(f, "Environment Yaml Parsing Error: {}", err)
            }
            RustamanError::UrlParseError(err) => write!(f, "Url Parse Error: {}", err),
            RustamanError::RequestParsingError(err) => write!(f, "{}", err),
            RustamanError::IOError(err) => write!(f, "{}", err),
            RustamanError::RenderError(err) => write!(f, "{}", err),
        }
    }
}

impl Error for RustamanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let err: Option<&(dyn Error + 'static)> = match self {
            RustamanError::EnvironmentParsingError(err) => Some(err),
            RustamanError::UrlParseError(err) => Some(err),
            RustamanError::IOError(err) => Some(err),
            _ => None,
        };
        err
    }
}

impl From<serde_yaml::Error> for RustamanError {
    fn from(err: serde_yaml::Error) -> RustamanError {
        RustamanError::EnvironmentParsingError(err)
    }
}

impl From<url::ParseError> for RustamanError {
    fn from(err: url::ParseError) -> RustamanError {
        RustamanError::UrlParseError(err)
    }
}

impl From<io::Error> for RustamanError {
    fn from(err: io::Error) -> RustamanError {
        RustamanError::IOError(err)
    }
}

impl From<handlebars::RenderError> for RustamanError {
    fn from(err: handlebars::RenderError) -> RustamanError {
        RustamanError::RenderError(err)
    }
}
