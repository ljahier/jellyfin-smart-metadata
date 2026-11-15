use std::{
    fmt,
    io,
};

#[derive(Debug)]
pub enum Error {
    UnknownCommand,
    MissingFlag {
        flag: &'static str,
    },
    MissingValue {
        flag: String,
    },
    UnknownFlag {
        flag: String,
    },
    Read {
        path: String,
        source: io::Error,
    },
    Write {
        path: String,
        source: io::Error,
    },
    Json {
        path: String,
        source: serde_json::Error,
    },
    TimestampRange {
        source: time::error::ComponentRange,
    },
    TimestampFormat {
        source: time::error::Format,
    },
}

impl fmt::Display for Error {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::UnknownCommand => write!(
                formatter,
                "Usage: xtask manifest --repository <owner/name> [--tag <tag>]."
            ),
            Self::MissingFlag {
                flag,
            } => write!(formatter, "Missing required flag {flag}."),
            Self::MissingValue {
                flag,
            } => write!(formatter, "Flag {flag} needs a value."),
            Self::UnknownFlag {
                flag,
            } => write!(formatter, "Unknown flag {flag}."),
            Self::Read {
                path,
                source,
            } => write!(formatter, "Cannot read {path}: {source}."),
            Self::Write {
                path,
                source,
            } => write!(formatter, "Cannot write {path}: {source}."),
            Self::Json {
                path,
                source,
            } => {
                write!(formatter, "Invalid JSON for {path}: {source}.")
            }
            Self::TimestampRange {
                source,
            } => {
                write!(formatter, "Cannot build the timestamp: {source}.")
            }
            Self::TimestampFormat {
                source,
            } => {
                write!(formatter, "Cannot format the timestamp: {source}.")
            }
        }
    }
}

impl std::error::Error for Error {}
