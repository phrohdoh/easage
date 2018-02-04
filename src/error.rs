use ::std::io;
use ::std::result;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Unable to find the path '{}'. Perhaps it does not exist or you do not have the required permissions.", path)]
    PathNotFound {
        path: String,
    },

    #[fail(display = "Unable to create an empty archive.")]
    AttemptCreateEmpty,

    #[fail(display = "Failed to read data from an incomplete archive.
Archive is {} bytes long but was expected to be at least {}.
Attempted to read from offset {:#X} to {:#X} inclusive.", actual_len, expected_len, read_start, read_end)]
    IncompleteArchive {
        actual_len: usize,
        expected_len: usize,
        read_start: usize,
        read_end: usize,
    },

    #[fail(display = "The requested entry does not exist in this archive.")]
    NoSuchEntry,

    #[fail(display = "I/O error: {}", inner)]
    IO {
        #[cause]
        inner: io::Error
    },

    #[fail(display = "The data provided {:?} is neither BIG4 nor BIGF.", magic)]
    InvalidMagic {
        magic: Vec<u8>,
    },

    #[fail(display = "{}", message)]
    Custom {
        message: String,
    },
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO {
            inner: e,
        }
    }
}

impl From<::walkdir::Error> for Error {
    fn from(e: ::walkdir::Error) -> Self {
        let path = e.path()
            .map(|ref_path| ref_path.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("<unknown path>"));

        Error::PathNotFound { path }
    }
}

pub type Result<T> = result::Result<T, Error>;