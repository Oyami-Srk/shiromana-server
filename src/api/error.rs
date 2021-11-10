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
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<LibError> for Error {
    fn from(err: LibError) -> Self {
        Self::LibraryError(err)
    }
}
