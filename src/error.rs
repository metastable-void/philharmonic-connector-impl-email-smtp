use philharmonic_connector_impl_api::ImplementationError;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub(crate) enum Error {
    #[error("{0}")]
    InvalidConfig(String),

    #[error("{0}")]
    InvalidRequest(String),

    #[error("upstream SMTP status {status}: {body}")]
    SmtpStatus { status: u16, body: String },

    #[error("{0}")]
    UpstreamUnreachable(String),

    #[error("{0}")]
    Internal(String),
}

impl From<Error> for ImplementationError {
    fn from(value: Error) -> Self {
        match value {
            Error::InvalidConfig(detail) => ImplementationError::InvalidConfig { detail },
            Error::InvalidRequest(detail) => ImplementationError::InvalidRequest { detail },
            Error::SmtpStatus { status, body } => {
                ImplementationError::UpstreamError { status, body }
            }
            Error::UpstreamUnreachable(detail) => {
                ImplementationError::UpstreamUnreachable { detail }
            }
            Error::Internal(detail) => ImplementationError::Internal { detail },
        }
    }
}
