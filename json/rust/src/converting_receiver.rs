//! A wrapper for [`Receiver`](std::sync::mpsc::Receiver), which converts received byte vectors to structured data.
/// Error type for interactions with Tinkerforge bricks or bricklets.
#[derive(Debug, Copy, Clone)]
pub enum BrickletError {
    /// A parameter was invalid or had an unexpected length
    InvalidParameter,
    /// The brick or bricklet does not support the requested function.
    FunctionNotSupported,
    /// Currently unused
    UnknownError,
    /// The request can not be fulfulled, as there is currently no connection to a brick daemon.
    NotConnected,
    /// The request was sent, but response expected is disabled, so no response can be received. This is not an error.
    SuccessButResponseExpectedIsDisabled,
}

impl From<u8> for BrickletError {
    fn from(byte: u8) -> BrickletError {
        match byte {
            1 => BrickletError::InvalidParameter,
            2 => BrickletError::FunctionNotSupported,
            _ => BrickletError::UnknownError,
        }
    }
}

impl std::fmt::Display for BrickletError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                BrickletError::InvalidParameter => "A parameter was invalid or had an unexpected length.",
                BrickletError::FunctionNotSupported => "The brick or bricklet does not support the requested function.",
                BrickletError::UnknownError => "UnknownError, Currently unused",
                BrickletError::NotConnected => "The request can not be fulfulled, as there is currently no connection to a brick daemon.",
                BrickletError::SuccessButResponseExpectedIsDisabled =>
                    "The request was sent, but response expected is disabled, so no response can be received. This is not an error.",
            }
        )
    }
}

impl std::error::Error for BrickletError {}

/// Error type which is returned if a ConvertingReceiver::recv call fails.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BrickletRecvTimeoutError {
    /// The queue was disconnected. This usually happens if the ip connection is destroyed.
    QueueDisconnected,
    /// The request could not be responded to before the timeout was reached.
    QueueTimeout,
    /// A parameter was invalid or had an unexpected length.
    InvalidParameter,
    /// The brick or bricklet does not support the requested function.
    FunctionNotSupported,
    /// Currently unused
    UnknownError,
    /// The received packet had an unexpected length. Maybe a function was called on a wrong brick or bricklet?
    MalformedPacket,
    /// The request can not be fulfulled, as there is currently no connection to a brick daemon.
    NotConnected,
    /// The request was sent, but response expected is disabled, so no response can be received. This is not an error.
    SuccessButResponseExpectedIsDisabled,
}

impl std::fmt::Display for BrickletRecvTimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                BrickletRecvTimeoutError::QueueDisconnected =>
                    "The queue was disconnected. This usually happens if the ip connection is destroyed.",
                BrickletRecvTimeoutError::QueueTimeout => "The request could not be responded to before the timeout was reached.",
                BrickletRecvTimeoutError::InvalidParameter => "A parameter was invalid or had an unexpected length.",
                BrickletRecvTimeoutError::FunctionNotSupported => "The brick or bricklet does not support the requested function.",
                BrickletRecvTimeoutError::UnknownError => "UnknownError, Currently unused",
                BrickletRecvTimeoutError::MalformedPacket =>
                    "The received packet had an unexpected length. Maybe a function was called on a wrong brick or bricklet?",
                BrickletRecvTimeoutError::NotConnected =>
                    "The request can not be fulfulled, as there is currently no connection to a brick daemon.",
                BrickletRecvTimeoutError::SuccessButResponseExpectedIsDisabled =>
                    "The request was sent, but response expected is disabled, so no response can be received. This is not an error.",
            }
        )
    }
}

impl std::error::Error for BrickletRecvTimeoutError {}

/// Error type which is returned if a [`try_recv`](crate::converting_receiver::ConvertingReceiver::try_recv) call fails.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BrickletTryRecvError {
    /// The queue was disconnected. This usually happens if the ip connection is destroyed.
    QueueDisconnected,
    /// There are currently no responses available.
    QueueEmpty,
    /// A parameter was invalid or had an unexpected length.
    InvalidParameter,
    /// The brick or bricklet does not support the requested function.
    FunctionNotSupported,
    /// Currently unused
    UnknownError,
    /// The received packet had an unexpected length. Maybe a function was called on a wrong brick or bricklet?
    MalformedPacket,
    /// The request can not be fulfulled, as there is currently no connection to a brick daemon.
    NotConnected,
    /// The request was sent, but response expected is disabled, so no response can be received. This is not an error.
    SuccessButResponseExpectedIsDisabled,
}

impl std::fmt::Display for BrickletTryRecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                BrickletTryRecvError::QueueDisconnected =>
                    "The queue was disconnected. This usually happens if the ip connection is destroyed.",
                BrickletTryRecvError::QueueEmpty => "There are currently no responses available.",
                BrickletTryRecvError::InvalidParameter => "A parameter was invalid or had an unexpected length.",
                BrickletTryRecvError::FunctionNotSupported => "The brick or bricklet does not support the requested function.",
                BrickletTryRecvError::UnknownError => "UnknownError, Currently unused",
                BrickletTryRecvError::MalformedPacket =>
                    "The received packet had an unexpected length. Maybe a function was called on a wrong brick or bricklet?",
                BrickletTryRecvError::NotConnected =>
                    "The request can not be fulfulled, as there is currently no connection to a brick daemon.",
                BrickletTryRecvError::SuccessButResponseExpectedIsDisabled =>
                    "The request was sent, but response expected is disabled, so no response can be received. This is not an error.",
            }
        )
    }
}

impl std::error::Error for BrickletTryRecvError {}
