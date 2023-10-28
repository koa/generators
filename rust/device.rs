//! Generic device functionality which is used by all bricks and bricklets.

use std::time::Duration;

use futures_core::Stream;

use crate::{
    base58::Base58,
    error::TinkerforgeError,
    ip_connection::async_io::{AsyncIpConnection, PacketData},
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum ResponseExpectedFlag {
    InvalidFunctionId,
    False,
    True,
    AlwaysTrue,
}

impl From<bool> for ResponseExpectedFlag {
    fn from(b: bool) -> Self {
        if b {
            ResponseExpectedFlag::True
        } else {
            ResponseExpectedFlag::False
        }
    }
}
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub(crate) struct Device {
    pub api_version: [u8; 3],
    pub response_expected: [ResponseExpectedFlag; 256],
    pub internal_uid: u32,
    pub connection: AsyncIpConnection,
}

/// This error is returned if the response expected status was queried for an unknown function.
#[derive(Debug, Copy, Clone)]
pub struct GetResponseExpectedError(u8);

impl std::error::Error for GetResponseExpectedError {}

impl std::fmt::Display for GetResponseExpectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Can not get response expected: Invalid function id {}",
            self.0
        )
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
            SetResponseExpectedError::InvalidFunctionId(fid) => write!(
                f,
                "Can not set response expected: Invalid function id {}",
                fid
            ),
            SetResponseExpectedError::IsAlwaysTrue(_fid) => write!(
                f,
                "Can not set response expected: function always responds."
            ),
        }
    }
}

impl Device {
    pub(crate) fn new(api_version: [u8; 3], uid: &str, connection: AsyncIpConnection) -> Device {
        match uid.base58_to_u32() {
            Ok(internal_uid) => Device {
                api_version,
                internal_uid,
                response_expected: [ResponseExpectedFlag::InvalidFunctionId; 256],
                connection,
            },
            //FIXME: (breaking change) Don't panic here, return a Result instead.
            Err(e) => panic!("UID {} could not be parsed: {}", uid, e),
        }
    }

    pub(crate) fn get_response_expected(
        &self,
        function_id: u8,
    ) -> Result<bool, GetResponseExpectedError> {
        match self.response_expected[function_id as usize] {
            ResponseExpectedFlag::False => Ok(false),
            ResponseExpectedFlag::True => Ok(true),
            ResponseExpectedFlag::AlwaysTrue => Ok(true),
            ResponseExpectedFlag::InvalidFunctionId => Err(GetResponseExpectedError(function_id)),
        }
    }

    pub(crate) fn set_response_expected(
        &mut self,
        function_id: u8,
        response_expected: bool,
    ) -> Result<(), SetResponseExpectedError> {
        if self.response_expected[function_id as usize] == ResponseExpectedFlag::AlwaysTrue {
            Err(SetResponseExpectedError::IsAlwaysTrue(function_id))
        } else if self.response_expected[function_id as usize]
            == ResponseExpectedFlag::InvalidFunctionId
        {
            Err(SetResponseExpectedError::InvalidFunctionId(function_id))
        } else {
            self.response_expected[function_id as usize] =
                ResponseExpectedFlag::from(response_expected);
            Ok(())
        }
    }

    pub(crate) fn set_response_expected_all(&mut self, response_expected: bool) {
        for resp_exp in self.response_expected.iter_mut() {
            if *resp_exp == ResponseExpectedFlag::True || *resp_exp == ResponseExpectedFlag::False {
                *resp_exp = ResponseExpectedFlag::from(response_expected);
            }
        }
    }

    pub(crate) async fn set(
        &mut self,
        function_id: u8,
        payload: &[u8],
    ) -> Result<Option<PacketData>, TinkerforgeError> {
        let timeout = if self.response_expected[function_id as usize] == ResponseExpectedFlag::False
        {
            None
        } else {
            Some(DEFAULT_TIMEOUT)
        };
        let result = self
            .connection
            .set(self.internal_uid, function_id, payload, timeout)
            .await?;
        Ok(result)
    }

    pub(crate) async fn get_callback_receiver(
        &mut self,
        function_id: u8,
    ) -> impl Stream<Item = PacketData> {
        self.connection
            .callback_stream(self.internal_uid, function_id)
            .await
    }

    pub(crate) async fn get(
        &mut self,
        function_id: u8,
        payload: &[u8],
    ) -> Result<PacketData, TinkerforgeError> {
        self.connection
            .get(self.internal_uid, function_id, payload, DEFAULT_TIMEOUT)
            .await
    }
}
