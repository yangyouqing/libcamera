/// Unified error type for the camera system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CamError {
    Ok = 0,
    InvalidParam = 1,
    Timeout = 2,
    BufferFull = 3,
    BufferEmpty = 4,
    NotFound = 5,
    PermissionDenied = 6,
    IoError = 7,
    NetworkError = 8,
    StorageError = 9,
    EncodingError = 10,
    ProtocolError = 11,
    AuthFailed = 12,
    NotReady = 13,
    Unsupported = 14,
    AlreadyExists = 15,
    ResourceExhausted = 16,
    ServiceDegraded = 17,
    ServiceSuspended = 18,
    InternalError = 255,
}

impl core::fmt::Display for CamError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Ok => write!(f, "Ok"),
            Self::InvalidParam => write!(f, "InvalidParam"),
            Self::Timeout => write!(f, "Timeout"),
            Self::BufferFull => write!(f, "BufferFull"),
            Self::BufferEmpty => write!(f, "BufferEmpty"),
            Self::NotFound => write!(f, "NotFound"),
            Self::PermissionDenied => write!(f, "PermissionDenied"),
            Self::IoError => write!(f, "IoError"),
            Self::NetworkError => write!(f, "NetworkError"),
            Self::StorageError => write!(f, "StorageError"),
            Self::EncodingError => write!(f, "EncodingError"),
            Self::ProtocolError => write!(f, "ProtocolError"),
            Self::AuthFailed => write!(f, "AuthFailed"),
            Self::NotReady => write!(f, "NotReady"),
            Self::Unsupported => write!(f, "Unsupported"),
            Self::AlreadyExists => write!(f, "AlreadyExists"),
            Self::ResourceExhausted => write!(f, "ResourceExhausted"),
            Self::ServiceDegraded => write!(f, "ServiceDegraded"),
            Self::ServiceSuspended => write!(f, "ServiceSuspended"),
            Self::InternalError => write!(f, "InternalError"),
        }
    }
}

pub type CommResult<T> = Result<T, CamError>;
