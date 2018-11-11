//! Define results and error. `Result<T, CabotError>`
use std::error::Error;
use std::fmt::{self, Display};

use gdk;
use serde_yaml;
use url;

#[derive(Debug)]
pub enum RustamanError {
    RequestParsingError(String),
    EnvironmentParsingError(serde_yaml::Error),
    UrlParseError(url::ParseError),
    GdkError(gdk::Error),
}

/// Result used by method that can failed.
pub type RustamanResult<T> = Result<T, RustamanError>;

impl Display for RustamanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = match self {
            RustamanError::EnvironmentParsingError(err) => {
                format!("Environment Yaml Parsing Error: {}", err)
            }
            RustamanError::UrlParseError(err) => format!("Url Parse Error: {}", err),
            RustamanError::RequestParsingError(err) => format!("{}", err),
            RustamanError::GdkError(err) => format!("{}", err),
        };
        write!(f, "{}", description)
    }
}

impl Error for RustamanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let err: Option<&(dyn Error + 'static)> = match self {
            RustamanError::EnvironmentParsingError(err) => Some(err),
            RustamanError::UrlParseError(err) => Some(err),
            RustamanError::GdkError(err) => Some(err),
            RustamanError::RequestParsingError(_) => None,
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

impl From<gdk::Error> for RustamanError {
    fn from(err: gdk::Error) -> RustamanError {
        RustamanError::GdkError(err)
    }
}
