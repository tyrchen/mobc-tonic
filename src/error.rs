use thiserror::Error;

/// General error definition
#[derive(Error, Debug)]
pub enum MobcTonicError {
    #[error("Connection timeout")]
    Timeout,
    #[error("Bad connection")]
    BadConn,
    // detailed errors
    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),
    #[error["GRPC error: {0}"]]
    GrpcError(#[from] tonic::Status),
    #[error["GRPC transport error: {0}"]]
    GrpcTransportError(#[from] tonic::transport::Error),
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
}
