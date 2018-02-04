use ::std::io;
use ::std::result;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Unable to find the path '{}'. Perhaps it does not exist or you do not have the required permissions.", path)]
    PathNotFound {
        path: String,
    },

    #[fail(display = "Unable to create an empty archive")]
    AttemptCreateEmpty,

    #[fail(display = "I/O error: {:?}", inner)]
    IO {
        #[cause]
        inner: io::Error
    },

    #[fail(display = "The data provided {:?} is neither BIG4 nor BIGF", magic)]
    InvalidMagic {
        magic: Vec<u8>,
    },

    #[fail(display = "{}", message)]
    Custom {
        message: String,
    },
}

pub type Result<T> = result::Result<T, Error>;