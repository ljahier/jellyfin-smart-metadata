use std::panic::{
    AssertUnwindSafe,
    catch_unwind,
};

use crate::movie_filename::{
    ParseError,
    ReleaseYear,
    parse_movie_filename,
};

#[repr(C)]
pub struct SmMovie {
    pub year: u16,
    pub title_len: usize,
}

enum Status {
    Success,
    NullPointer,
    InvalidUtf8,
    Empty,
    TooLong,
    NoTitle,
    TitleBufferTooSmall,
    Panic,
}

impl Status {
    fn code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::NullPointer => 1,
            Self::InvalidUtf8 => 2,
            Self::Empty => 3,
            Self::TooLong => 4,
            Self::NoTitle => 5,
            Self::TitleBufferTooSmall => 6,
            Self::Panic => 7,
        }
    }
}

impl From<ParseError> for Status {
    fn from(error: ParseError) -> Self {
        match error {
            ParseError::Empty => Self::Empty,
            ParseError::TooLong {
                ..
            } => Self::TooLong,
            ParseError::NoTitle => Self::NoTitle,
        }
    }
}

#[expect(
    clippy::missing_safety_doc,
    reason = "Exported C symbol whose only caller is the C# adapter."
)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_parse_movie_filename(
    file_name: *const u8,
    file_name_len: usize,
    title_buffer: *mut u8,
    title_buffer_len: usize,
    movie_out: *mut SmMovie,
) -> i32 {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: the caller upholds the pointer and length contract of this
        // function.
        unsafe {
            parse_into(
                file_name,
                file_name_len,
                title_buffer,
                title_buffer_len,
                movie_out,
            )
        }
    }));
    let status = match outcome {
        Ok(Ok(())) => Status::Success,
        Ok(Err(status)) => status,
        Err(_) => Status::Panic,
    };
    status.code()
}

unsafe fn parse_into(
    file_name: *const u8,
    file_name_len: usize,
    title_buffer: *mut u8,
    title_buffer_len: usize,
    movie_out: *mut SmMovie,
) -> Result<(), Status> {
    if file_name.is_null() || title_buffer.is_null() || movie_out.is_null() {
        return Err(Status::NullPointer);
    }

    // SAFETY: file_name is non-null and the caller guarantees file_name_len
    // readable bytes.
    let file_name_bytes = unsafe { std::slice::from_raw_parts(file_name, file_name_len) };
    let file_name_text = std::str::from_utf8(file_name_bytes).map_err(|_| Status::InvalidUtf8)?;
    let movie = parse_movie_filename(file_name_text)?;

    let title = movie.title.as_bytes();
    // SAFETY: title_buffer is non-null and the caller guarantees
    // title_buffer_len writable bytes.
    let title_destination =
        unsafe { std::slice::from_raw_parts_mut(title_buffer, title_buffer_len) };
    title_destination
        .get_mut(..title.len())
        .ok_or(Status::TitleBufferTooSmall)?
        .copy_from_slice(title);

    let year = movie.year.map_or(0, ReleaseYear::get);
    // SAFETY: movie_out is non-null and the caller guarantees it points to a
    // writable SmMovie.
    unsafe {
        movie_out.write(SmMovie {
            year,
            title_len: title.len(),
        })
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        SmMovie,
        sm_parse_movie_filename,
    };

    const TITLE_BUFFER_BYTES: usize = 64;

    #[derive(Debug, PartialEq, Eq)]
    struct Call {
        status: i32,
        year: u16,
        title: String,
    }

    fn call(
        file_name: &[u8],
        title_buffer_len: usize,
    ) -> Call {
        let mut title_buffer = vec![0_u8; title_buffer_len];
        let mut movie = SmMovie {
            year: 0,
            title_len: 0,
        };
        // SAFETY: every pointer comes from a live Rust value and every length
        // matches its allocation.
        let status = unsafe {
            sm_parse_movie_filename(
                file_name.as_ptr(),
                file_name.len(),
                title_buffer.as_mut_ptr(),
                title_buffer.len(),
                &raw mut movie,
            )
        };
        let title = title_buffer
            .get(..movie.title_len)
            .map_or_else(String::new, |bytes| {
                String::from_utf8_lossy(bytes).into_owned()
            });
        Call {
            status,
            year: movie.year,
            title,
        }
    }

    #[test]
    fn reports_title_and_year_for_a_name_jellyfin_misreads() {
        assert_eq!(
            call(
                b"2.Finding.Nemo.2003.MULTI.2160p.BluRay.HDR.REMUX.HEVC.TRUEHD.Atmos-GROUP .mkv",
                TITLE_BUFFER_BYTES
            ),
            Call {
                status: 0,
                year: 2003,
                title: String::from("Finding Nemo"),
            },
            "The index prefix and the technical tokens must be stripped."
        );
    }

    #[test]
    fn reports_year_zero_when_the_name_has_none() {
        assert_eq!(
            call(b"Paddington.1080p.WEB-DL.mkv", TITLE_BUFFER_BYTES),
            Call {
                status: 0,
                year: 0,
                title: String::from("Paddington"),
            },
            "A name without a year must report year 0."
        );
    }

    #[test]
    fn rejects_invalid_utf8() {
        assert_eq!(
            call(&[0xff, 0xfe], TITLE_BUFFER_BYTES).status,
            2,
            "Bytes that are not UTF-8 must be reported."
        );
    }

    #[test]
    fn rejects_an_empty_name() {
        assert_eq!(
            call(b"", TITLE_BUFFER_BYTES).status,
            3,
            "An empty name must be reported."
        );
    }

    #[test]
    fn rejects_a_name_over_the_length_limit() {
        assert_eq!(
            call(&[b'a'; 4097], TITLE_BUFFER_BYTES).status,
            4,
            "A name over the limit must be reported."
        );
    }

    #[test]
    fn rejects_a_name_made_only_of_technical_tokens() {
        assert_eq!(
            call(b"1080p.BluRay.mkv", TITLE_BUFFER_BYTES).status,
            5,
            "Technical tokens alone must be reported as no title."
        );
    }

    #[test]
    fn rejects_a_title_buffer_that_is_too_small() {
        assert_eq!(
            call(b"Finding.Nemo.2003.mkv", 3).status,
            6,
            "A buffer shorter than the title must be reported."
        );
    }

    #[test]
    fn rejects_a_null_file_name_pointer() {
        let mut title_buffer = [0_u8; 8];
        let mut movie = SmMovie {
            year: 0,
            title_len: 0,
        };
        // SAFETY: the null file name is rejected before any read; the other
        // pointers come from live values.
        let status = unsafe {
            sm_parse_movie_filename(
                std::ptr::null(),
                0,
                title_buffer.as_mut_ptr(),
                title_buffer.len(),
                &raw mut movie,
            )
        };
        assert_eq!(status, 1, "A null input pointer must be reported.");
    }
}
