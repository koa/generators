//! Generic device functionality which is used by all bricks and bricklets.

use std::time::Duration;

use futures_core::Stream;
#[cfg(feature = "prometheus")]
use lazy_static::lazy_static;
#[cfg(feature = "prometheus")]
use prometheus::{register_histogram_vec, HistogramVec};

use crate::{
    base58::Uid,
    error::TinkerforgeError,
    ip_connection::async_io::{AsyncIpConnection, PacketData},
};

#[cfg(feature = "prometheus")]
lazy_static! {
    static ref REQUEST_TIMING: HistogramVec = register_histogram_vec!(
        "tinkerforge_request",
        "The Tinkerforge response times latencies in seconds.",
        &["device_type", "function_id", "method"]
    )
    .unwrap();
}

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Debug)]
pub(crate) struct Device {
    pub internal_uid: Uid,
    pub connection: AsyncIpConnection,
    #[cfg(feature = "prometheus")]
    device_display_name: &'static str,
}

/// This error is returned if the response expected status was queried for an unknown function.
#[derive(Debug, Copy, Clone)]
pub struct GetResponseExpectedError(u8);

impl std::error::Error for GetResponseExpectedError {}

impl std::fmt::Display for GetResponseExpectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Can not get response expected: Invalid function id {}", self.0)
    }
}

/// This error is returned if the response expected status of a function could not be changed.
#[derive(Debug, Copy, Clone)]
pub enum SetResponseExpectedError {
    /// The function id was unknown. Maybe the wrong UID was used?
    InvalidFunctionId(u8),
    /// This function must always respond, as the response contains result data.
    IsAlwaysTrue(u8),
}

impl std::error::Error for SetResponseExpectedError {}

impl std::fmt::Display for SetResponseExpectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SetResponseExpectedError::InvalidFunctionId(fid) => write!(f, "Can not set response expected: Invalid function id {}", fid),
            SetResponseExpectedError::IsAlwaysTrue(_fid) => write!(f, "Can not set response expected: function always responds."),
        }
    }
}

impl Device {
    pub(crate) fn new(internal_uid: Uid, connection: AsyncIpConnection, #[allow(unused)] device_display_name: &'static str) -> Device {
        Device {
            internal_uid,
            connection,
            #[cfg(feature = "prometheus")]
            device_display_name,
        }
    }

    pub(crate) async fn set(
        &mut self,
        function_id: u8,
        payload: &[u8],
        timeout: Option<Duration>,
    ) -> Result<Option<PacketData>, TinkerforgeError> {
        #[cfg(feature = "prometheus")]
        let timer = REQUEST_TIMING.with_label_values(&[self.device_display_name, function_id.to_string().as_str(), "set"]).start_timer();
        let result = self.connection.set(self.internal_uid, function_id, payload, timeout).await;
        #[cfg(feature = "prometheus")]
        drop(timer);
        result
    }

    pub(crate) async fn get_callback_receiver(&mut self, function_id: u8) -> impl Stream<Item = PacketData> {
        self.connection.callback_stream(self.internal_uid, function_id).await
    }

    pub(crate) async fn get(&mut self, function_id: u8, payload: &[u8]) -> Result<PacketData, TinkerforgeError> {
        #[cfg(feature = "prometheus")]
        let timer = REQUEST_TIMING.with_label_values(&[self.device_display_name, function_id.to_string().as_str(), "get"]).start_timer();
        let result = self.connection.get(self.internal_uid, function_id, payload, DEFAULT_TIMEOUT).await;
        #[cfg(feature = "prometheus")]
        drop(timer);
        result
    }
}
