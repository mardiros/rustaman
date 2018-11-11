//! Define results and error. `Result<T, CabotError>`
use std::error::Error;
use std::fmt::{self, Display};
use serde_yaml;


#[derive(Debug)]
pub enum RustamanError{
    EnvironmentParsingError(serde_yaml::Error)
}


impl Display for RustamanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = match self {
            RustamanError::EnvironmentParsingError(err) => format!("Environment Yaml Parsing Error: {}", err),
        };
        write!(f, "{}", description)
    }
}

impl Error for RustamanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let err: Option<&(dyn Error + 'static)> = match self {
            RustamanError::EnvironmentParsingError(err) => Some(err),
        };
        err
    }
}


impl From<serde_yaml::Error> for RustamanError {
    fn from(err: serde_yaml::Error) -> RustamanError {
        RustamanError::EnvironmentParsingError(err)
    }
}
