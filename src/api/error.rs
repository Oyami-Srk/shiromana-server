use shiromana_rs::misc::Uuid;

pub enum Error {
    NotExisted {
        got: String,
        field: String,
        expect: String,
    },
    LibraryNotOpened(Uuid),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotExisted { got, field, expect } => write!(
                f,
                "Field {}: `{}` is not exists on the disk of server. Or it is not a {}.",
                field, got, expect
            ),
            Self::LibraryNotOpened(lib) => write!(f, "Library `{}` is not opened.", lib),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
