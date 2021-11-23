use shiromana_rs::{library::Library, misc::Error as LibError, misc::Uuid};

pub enum Error {
    NotExisted {
        got: String,
        field: String,
        expect: String,
    },
    LibraryNotOpened(Uuid),
    NoParam(String),
    ParamInvalid {
        got: String,
        field: String,
        expect: String,
    },
    AlreadyExisted {
        got: String,
        field: String,
    },
    LibraryError(LibError),
    IOError(std::io::Error),
    SerializeError(serde_json::Error),
    MultithreadError(Box<dyn std::error::Error + Sync + Send>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotExisted { got, field, expect } => write!(
                f,
                "Field {}: `{}` is not existed on the disk of server. Or it is not a {}.",
                field, got, expect
            ),
            Self::AlreadyExisted { got, field } => write!(
                f,
                "Field {}: `{}` is already existed on the disk of server.",
                field, got
            ),
            Self::LibraryNotOpened(lib) => write!(f, "Library `{}` is not opened.", lib),
            Self::NoParam(what) => write!(f, "Params {} not provided.", what),
            Self::ParamInvalid { got, field, expect } => write!(
                f,
                "Param `{}` with value `{}` cannot be parsed to `{}`.",
                field, got, expect
            ),
            Self::LibraryError(err) => write!(f, "Library Error: {}", err),
            Self::IOError(err) => write!(f, "IO Error: {}", err),
            Self::SerializeError(err) => write!(f, "Serialize Error: {}", err),
            Self::MultithreadError(err) => write!(f, "Multithrad Error: {}", err)
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<LibError> for Error {
    fn from(err: LibError) -> Self {
        Self::LibraryError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializeError(err)
    }
}

impl From<std::sync::mpsc::RecvError> for Error {
    fn from(err: std::sync::mpsc::RecvError) -> Self {
        Self::MultithreadError(Box::new(err))
    }
}