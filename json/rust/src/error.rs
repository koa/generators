use std::{
    array::TryFromSliceError,
    io
};
use thiserror::Error;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

#[derive(Error, Debug)]
pub enum TinkerforgeError {
    #[error("IO Error: {0}")]
    IoError(#[from] io::Error),
    #[error("No Response")]
    NoResponseReceived,
    #[error("Error receiving data from broadcast stream: {0}")]
    BroadcastStreamRecvError(#[from] BroadcastStreamRecvError),
    #[error("Cannot extract slice from Packet")]
    PackedDecodingError(#[from] TryFromSliceError),
}
