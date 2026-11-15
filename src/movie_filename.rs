use std::fmt;

const FILE_NAME_BYTES_MAX: usize = 4096;
const RELEASE_YEAR_MIN: u16 = 1900;
const RELEASE_YEAR_MAX: u16 = 2100;

const VIDEO_EXTENSIONS: &[&str] = &["mkv", "mp4", "avi", "m4v", "mov", "wmv", "ts"];

const NOISE_TOKENS: &[&str] = &[
    "4k",
    "uhd",
    "bluray",
    "brrip",
    "bdrip",
    "webrip",
    "web-dl",
    "hdtv",
    "dvdrip",
    "remux",
    "hdr",
    "hdr10",
    "hevc",
    "x264",
    "x265",
    "h264",
    "h265",
    "dts",
    "truehd",
    "atmos",
    "aac",
    "ac3",
    "multi",
    "vff",
    "vfq",
    "theatrical",
    "extended",
    "unrated",
    "remastered",
    "imax",
    "proper",
    "repack",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseYear(u16);

impl ReleaseYear {
    pub fn get(self) -> u16 { self.0 }
}

#[derive(Debug, PartialEq, Eq)]
pub struct YearOutOfRange {
    pub value: u16,
}

impl TryFrom<u16> for ReleaseYear {
    type Error = YearOutOfRange;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if (RELEASE_YEAR_MIN..=RELEASE_YEAR_MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(YearOutOfRange {
                value,
            })
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParsedMovie {
    pub title: String,
    pub year: Option<ReleaseYear>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    TooLong { bytes: usize, bytes_max: usize },
    NoTitle,
}

impl fmt::Display for ParseError {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::Empty => write!(formatter, "File name is empty."),
            Self::TooLong {
                bytes,
                bytes_max,
            } => write!(
                formatter,
                "File name is {bytes} bytes long, the limit is {bytes_max}."
            ),
            Self::NoTitle => write!(formatter, "File name contains no title."),
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse_movie_filename(file_name: &str) -> Result<ParsedMovie, ParseError> {
    let bytes = file_name.len();
    if bytes > FILE_NAME_BYTES_MAX {
        return Err(ParseError::TooLong {
            bytes,
            bytes_max: FILE_NAME_BYTES_MAX,
        });
    }

    let trimmed = file_name.trim();
    if trimmed.is_empty() {
        return Err(ParseError::Empty);
    }

    let base = strip_group_prefix(strip_extension(trimmed));
    let all_tokens = tokenize(base);
    let tokens = strip_index_prefix(&all_tokens);

    let noise_start = tokens
        .iter()
        .position(|token| is_noise(token))
        .unwrap_or(tokens.len());
    let placement = locate_year(tokens, noise_start);

    let title = tokens
        .iter()
        .take(placement.title_token_count)
        .copied()
        .collect::<Vec<&str>>()
        .join(" ");
    if title.is_empty() {
        return Err(ParseError::NoTitle);
    }

    Ok(ParsedMovie {
        title,
        year: placement.year,
    })
}

struct YearCandidate {
    token_index: usize,
    year: ReleaseYear,
}

struct YearPlacement {
    title_token_count: usize,
    year: Option<ReleaseYear>,
}

fn tokenize(base: &str) -> Vec<&str> {
    base.split(['.', '_', ' '])
        .map(|token| token.trim_matches(['(', ')', '[', ']']))
        .filter(|token| !token.is_empty() && *token != "-")
        .collect()
}

fn strip_index_prefix<'input>(tokens: &'input [&'input str]) -> &'input [&'input str] {
    match tokens.split_first() {
        Some((first, rest)) if is_index_prefix(first) && !rest.is_empty() => rest,
        Some(_) | None => tokens,
    }
}

fn locate_year(
    tokens: &[&str],
    noise_start: usize,
) -> YearPlacement {
    let candidates: Vec<YearCandidate> = tokens
        .iter()
        .enumerate()
        .skip(1)
        .filter_map(|(token_index, token)| {
            parse_year(token).map(|year| YearCandidate {
                token_index,
                year,
            })
        })
        .collect();

    match candidates
        .iter()
        .rev()
        .find(|candidate| candidate.token_index < noise_start)
    {
        Some(candidate) => YearPlacement {
            title_token_count: candidate.token_index,
            year: Some(candidate.year),
        },
        None => YearPlacement {
            title_token_count: noise_start,
            year: candidates.first().map(|candidate| candidate.year),
        },
    }
}

fn strip_extension(name: &str) -> &str {
    name.rsplit_once('.')
        .filter(|(_, extension)| {
            VIDEO_EXTENSIONS
                .iter()
                .any(|known| known.eq_ignore_ascii_case(extension))
        })
        .map_or(name, |(stem, _)| stem)
}

fn strip_group_prefix(name: &str) -> &str {
    name.trim_start()
        .strip_prefix('[')
        .and_then(|rest| rest.split_once(']'))
        .map_or(name, |(_, rest)| rest)
}

fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

fn before_hyphen(token: &str) -> &str { token.split('-').next().unwrap_or_default() }

fn is_index_prefix(token: &str) -> bool { (1..=2).contains(&token.len()) && is_digits(token) }

fn is_noise(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();
    [lower.as_str(), before_hyphen(&lower)]
        .into_iter()
        .any(|candidate| NOISE_TOKENS.contains(&candidate) || is_resolution(candidate))
}

fn is_resolution(lower: &str) -> bool {
    lower
        .strip_suffix('p')
        .is_some_and(|digits| (3..=4).contains(&digits.len()) && is_digits(digits))
}

fn parse_year(token: &str) -> Option<ReleaseYear> {
    let digits = before_hyphen(token);
    if digits.len() != 4 || !is_digits(digits) {
        return None;
    }

    digits
        .parse::<u16>()
        .ok()
        .and_then(|value| ReleaseYear::try_from(value).ok())
}

#[cfg(test)]
mod tests {
    use super::{
        ParseError,
        ReleaseYear,
        YearOutOfRange,
        parse_movie_filename,
    };

    #[derive(Debug, PartialEq, Eq)]
    struct Identification {
        title: String,
        year: Option<u16>,
    }

    fn identify(file_name: &str) -> Result<Identification, ParseError> {
        let movie = parse_movie_filename(file_name)?;
        Ok(Identification {
            title: movie.title,
            year: movie.year.map(ReleaseYear::get),
        })
    }

    #[test]
    fn identifies_unconventional_finding_nemo_names() -> Result<(), ParseError> {
        let file_names = [
            "Finding.Nemo.2003.MULTI.2160p.BluRay.HDR.REMUX.HEVC.TRUEHD.Atmos-GROUP .mkv",
            "2.Finding.Nemo.2003.MULTI.2160p.BluRay.HDR.REMUX.HEVC.TRUEHD.Atmos-GROUP .mkv",
            "Finding.Nemo.Theatrical.Cut.2160p.UHD.BluRay.REMUX.HDR.HEVC.Multi.2003-GROUP .mkv",
            "[GRP] Finding_Nemo_-_2003_-_2160p_REMUX.mkv",
            "Finding Nemo (2003)",
        ];

        for file_name in file_names {
            assert_eq!(
                identify(file_name)?,
                Identification {
                    title: String::from("Finding Nemo"),
                    year: Some(2003),
                },
                "Unexpected result for {file_name}."
            );
        }
        Ok(())
    }

    #[test]
    fn keeps_numbers_that_belong_to_the_title() -> Result<(), ParseError> {
        assert_eq!(
            identify("2001.A.Space.Odyssey.1968.1080p.BluRay.mkv")?,
            Identification {
                title: String::from("2001 A Space Odyssey"),
                year: Some(1968),
            },
            "A year-like first token must stay in the title."
        );
        assert_eq!(
            identify("Blade.Runner.2049.2017.2160p.mkv")?,
            Identification {
                title: String::from("Blade Runner 2049"),
                year: Some(2017),
            },
            "The last year before the technical tokens is the release year."
        );
        Ok(())
    }

    #[test]
    fn drops_the_dot_after_an_abbreviation() -> Result<(), ParseError> {
        assert_eq!(
            identify("Fantastic.Mr.Fox.2009.1080p.BluRay.x264.mkv")?,
            Identification {
                title: String::from("Fantastic Mr Fox"),
                year: Some(2009),
            },
            "Dots are separators, including the one after an abbreviation."
        );
        Ok(())
    }

    #[test]
    fn returns_no_year_when_the_name_has_none() -> Result<(), ParseError> {
        assert_eq!(
            identify("Paddington.1080p.WEB-DL.mkv")?,
            Identification {
                title: String::from("Paddington"),
                year: None,
            },
            "A name without a year must still yield its title."
        );
        Ok(())
    }

    #[test]
    fn rejects_empty_names() {
        assert_eq!(
            parse_movie_filename(""),
            Err(ParseError::Empty),
            "An empty name has no title."
        );
        assert_eq!(
            parse_movie_filename("   "),
            Err(ParseError::Empty),
            "A blank name has no title."
        );
    }

    #[test]
    fn rejects_names_made_only_of_technical_tokens() {
        assert_eq!(
            parse_movie_filename("1080p.BluRay.mkv"),
            Err(ParseError::NoTitle),
            "Technical tokens alone are not a title."
        );
    }

    #[test]
    fn rejects_names_over_the_length_limit() {
        assert_eq!(
            parse_movie_filename(&"a".repeat(4097)),
            Err(ParseError::TooLong {
                bytes: 4097,
                bytes_max: 4096
            }),
            "A name over the limit must be rejected before parsing."
        );
    }

    #[test]
    fn bounds_the_release_year() {
        assert_eq!(
            ReleaseYear::try_from(1899),
            Err(YearOutOfRange {
                value: 1899
            }),
            "1899 is below the lower bound."
        );
        assert_eq!(
            ReleaseYear::try_from(2101),
            Err(YearOutOfRange {
                value: 2101
            }),
            "2101 is above the upper bound."
        );
        assert_eq!(
            ReleaseYear::try_from(1900).map(ReleaseYear::get),
            Ok(1900),
            "1900 is the lower bound."
        );
        assert_eq!(
            ReleaseYear::try_from(2100).map(ReleaseYear::get),
            Ok(2100),
            "2100 is the upper bound."
        );
    }
}
