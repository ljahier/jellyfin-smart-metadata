use std::fs;
use std::process::ExitCode;

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::error::Error;
use crate::manifest::{
    PluginMeta,
    Release,
    build_manifest,
    zip_file_name,
};

mod error;
mod manifest;

const META_PATH: &str = "plugin/meta.json";
const OUTPUT_PATH: &str = "dist/manifest.json";

struct Arguments {
    repository: String,
    tag: Option<String>,
}

impl Arguments {
    fn parse(mut arguments: impl Iterator<Item = String>) -> Result<Self, Error> {
        if arguments.next().as_deref() != Some("manifest") {
            return Err(Error::UnknownCommand);
        }

        let mut repository = None;
        let mut tag = None;
        while let Some(flag) = arguments.next() {
            let Some(value) = arguments.next() else {
                return Err(Error::MissingValue {
                    flag,
                });
            };
            match flag.as_str() {
                "--repository" => repository = Some(value),
                "--tag" => tag = Some(value),
                _ => {
                    return Err(Error::UnknownFlag {
                        flag,
                    });
                }
            }
        }

        let repository = repository.ok_or(Error::MissingFlag {
            flag: "--repository",
        })?;
        Ok(Self {
            repository,
            tag,
        })
    }
}

fn main() -> ExitCode {
    match run(std::env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: impl Iterator<Item = String>) -> Result<(), Error> {
    let arguments = Arguments::parse(arguments)?;

    let meta_text = fs::read_to_string(META_PATH).map_err(|source| Error::Read {
        path: String::from(META_PATH),
        source,
    })?;
    let meta = serde_json::from_str::<PluginMeta>(&meta_text).map_err(|source| Error::Json {
        path: String::from(META_PATH),
        source,
    })?;

    let tag = arguments
        .tag
        .unwrap_or_else(|| format!("v{}", meta.version));
    let zip_path = format!("dist/{}", zip_file_name(&meta.version));
    let zip = fs::read(&zip_path).map_err(|source| Error::Read {
        path: zip_path,
        source,
    })?;

    let release = Release {
        repository: arguments.repository,
        tag,
    };
    let manifest = build_manifest(meta, &release, &zip, timestamp_now()?);

    let json = serde_json::to_string_pretty(&manifest).map_err(|source| Error::Json {
        path: String::from(OUTPUT_PATH),
        source,
    })?;
    fs::write(OUTPUT_PATH, json).map_err(|source| Error::Write {
        path: String::from(OUTPUT_PATH),
        source,
    })
}

fn timestamp_now() -> Result<String, Error> {
    OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .map_err(|source| Error::TimestampRange {
            source,
        })?
        .format(&Rfc3339)
        .map_err(|source| Error::TimestampFormat {
            source,
        })
}
