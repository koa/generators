//! The IP Connection manages the communication between the API bindings and the Brick Daemon or a WIFI/Ethernet Extension.
use crate::base58::Uid;
use crate::bindings::DeviceIdentifier;
use std::str;

use crate::byte_converter::{FromByteSlice, ParsedOrRaw, ToBytes};

pub mod async_io {
    use std::{
        borrow::BorrowMut,
        fmt::Debug,
        ops::{Deref, DerefMut},
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        time::Duration,
    };

    use log::{debug, error, info, warn};
    use tokio::{
        io::{self, AsyncReadExt, AsyncWriteExt, WriteHalf},
        net::{TcpStream, ToSocketAddrs},
        sync::{
            broadcast::{self, Receiver},
            Mutex,
        },
        task::AbortHandle,
    };
    use tokio_stream::{
        empty,
        wrappers::{errors::BroadcastStreamRecvError, BroadcastStream},
        Stream, StreamExt,
    };

    use crate::{
        base58::Uid,
        byte_converter::{FromByteSlice, ToBytes},
        error::TinkerforgeError,
        ip_connection::EnumerationType,
        ip_connection::{EnumerateResponse, PacketHeader},
    };

    #[derive(Debug, Clone)]
    pub struct AsyncIpConnection {
        inner: Arc<Mutex<InnerAsyncIpConnection>>,
    }

    impl AsyncIpConnection {
        pub async fn enumerate(&mut self) -> Result<Box<dyn Stream<Item = EnumerateResponse> + Unpin + Send>, TinkerforgeError> {
            self.inner.borrow_mut().lock().await.enumerate().await
        }
        pub async fn disconnect_probe(&mut self) -> Result<(), TinkerforgeError> {
            self.inner.borrow_mut().lock().await.disconnect_probe().await
        }
        pub async fn get_authentication_nonce(&mut self) -> Result<[u8; 4], TinkerforgeError> {
            self.inner.borrow_mut().lock().await.get_authentication_nonce().await
        }
        pub(crate) async fn set(
            &mut self,
            uid: Uid,
            function_id: u8,
            payload: &[u8],
            timeout: Option<Duration>,
        ) -> Result<Option<PacketData>, TinkerforgeError> {
            self.inner.borrow_mut().lock().await.set(uid, function_id, payload, timeout).await
        }
        pub(crate) async fn get(
            &mut self,
            uid: Uid,
            function_id: u8,
            payload: &[u8],
            timeout: Duration,
        ) -> Result<PacketData, TinkerforgeError> {
            self.inner.borrow_mut().lock().await.get(uid, function_id, payload, timeout).await
        }
        pub(crate) async fn callback_stream(&mut self, uid: Uid, function_id: u8) -> impl Stream<Item = PacketData> {
            self.inner.borrow_mut().lock().await.callback_stream(uid, function_id).await
        }
    }

    impl AsyncIpConnection {
        pub async fn new<T: ToSocketAddrs + Debug + Clone + Send + 'static>(addr: T) -> Result<Self, TinkerforgeError> {
            Ok(Self { inner: Arc::new(Mutex::new(InnerAsyncIpConnection::new(addr).await?)) })
        }
    }

    #[derive(Debug)]
    struct InnerAsyncIpConnection {
        write_stream: WriteHalf<TcpStream>,
        receiver: Receiver<Option<PacketData>>,
        seq_num: u8,
        running: Arc<AtomicBool>,
        abort_handle: AbortHandle,
    }

    impl InnerAsyncIpConnection {
        pub async fn new<T: ToSocketAddrs + Clone + Debug + Send + 'static>(addr: T) -> Result<Self, TinkerforgeError> {
            let socket = TcpStream::connect(addr.clone()).await?;
            Self::enable_keepalive(&socket)?;

            let (mut rd, write_stream) = io::split(socket);
            let (enum_tx, receiver) = broadcast::channel(512);
            let running = Arc::new(AtomicBool::new(true));
            let running_clone = running.clone();
            let abort_handle = tokio::spawn(async move {
                loop {
                    let mut header_buffer = Box::new([0; PacketHeader::SIZE]);
                    match rd.read_exact(header_buffer.deref_mut()).await {
                        Ok(8) => {
                            let header = PacketHeader::from_le_byte_slice(header_buffer.deref());
                            let body_size = header.length as usize - PacketHeader::SIZE;
                            let mut body = vec![0; body_size].into_boxed_slice();
                            match rd.read_exact(body.deref_mut()).await {
                                Ok(l) if l == body_size => {}
                                Ok(l) => {
                                    panic!("Unexpected body size: {}", l)
                                }
                                Err(e) => panic!("Error from socket: {}", e),
                            }
                            let packet_data = PacketData { header, body };
                            debug!("Received: {packet_data:?}");
                            if let Err(error) = enum_tx.send(Some(packet_data)) {
                                warn!("Cannot process packet from {addr:?}: {error}");
                                break;
                            }
                        }
                        Ok(n) => {
                            error!("Unexpected read count from {addr:?}: {}", n);
                            if let Err(error) = enum_tx.send(None) {
                                warn!("Cannot close connection on read error: {error}");
                            }
                            break;
                        }
                        Err(e) => {
                            error!("Error from socket {addr:?}: {e}");
                            if let Err(error) = enum_tx.send(None) {
                                warn!("Cannot close connection on communication error: {error}");
                            }
                            break;
                        }
                    };
                }
                running_clone.store(false, Ordering::Relaxed);
                info!("Terminated receiver thread");
            })
            .abort_handle();
            Ok(Self { write_stream, abort_handle, seq_num: 1, receiver, running })
        }

        fn enable_keepalive(socket: &TcpStream) -> Result<(), TinkerforgeError> {
            let mut ka = socket2::TcpKeepalive::new();
            ka = ka.with_time(Duration::from_secs(20));
            ka = ka.with_interval(Duration::from_secs(20));
            socket2::SockRef::from(&socket).set_tcp_keepalive(&ka)?;
            Ok(())
        }
        pub async fn enumerate(&mut self) -> Result<Box<dyn Stream<Item = EnumerateResponse> + Unpin + Send>, TinkerforgeError> {
            if !self.running.as_ref().load(Ordering::Relaxed) {
                return Ok(Box::new(empty()));
            }
            let request = Request::Set { uid: Uid::zero(), function_id: 254, payload: &[] };
            let stream = BroadcastStream::new(self.receiver.resubscribe()).map_while(Self::while_some).filter_map(|p| match p {
                Ok(p) if p.header.function_id == 253 => Some(EnumerateResponse::from_le_byte_slice(&p.body)),
                _ => None,
            });
            let seq = self.next_seq();
            self.send_packet(&request, seq, true).await?;
            Ok(Box::new(stream))
        }
        pub async fn disconnect_probe(&mut self) -> Result<(), TinkerforgeError> {
            let request = Request::Set { uid: Uid::zero(), function_id: 128, payload: &[] };
            let seq = self.next_seq();
            self.send_packet(&request, seq, true).await?;
            Ok(())
        }
        async fn get_authentication_nonce(&mut self) -> Result<[u8; 4], TinkerforgeError> {
            let request = Request::Set { uid: Uid::zero(), function_id: 1, payload: &[] };
            let seq = self.next_seq();
            let stream = BroadcastStream::new(self.receiver.resubscribe()).map_while(Self::while_some).timeout(Duration::from_secs(5));
            self.send_packet(&request, seq, true).await?;
            tokio::pin!(stream);
            let option = stream.next().await;
            info!("Paket: {option:?}");
            if let Some(Ok(Ok(next_paket))) = option {
                let body = next_paket.body;
                if body.len() == 4 {
                    let mut ret = [0; 4];
                    ret.copy_from_slice(&body);
                    Ok(ret)
                } else {
                    todo!()
                }
            } else {
                todo!()
            }
        }
        pub async fn set(
            &mut self,
            uid: Uid,
            function_id: u8,
            payload: &[u8],
            timeout: Option<Duration>,
        ) -> Result<Option<PacketData>, TinkerforgeError> {
            let request = Request::Set { uid, function_id, payload };
            let seq = self.next_seq();
            if let Some(timeout) = timeout {
                let stream = BroadcastStream::new(self.receiver.resubscribe())
                    .map_while(Self::while_some)
                    .filter(Self::filter_response(uid, function_id, seq))
                    .timeout(timeout);
                self.send_packet(&request, seq, true).await?;
                tokio::pin!(stream);
                if let Some(done) = stream.next().await {
                    Ok(Some(done.map_err(|_| TinkerforgeError::NoResponseReceived)??))
                } else {
                    Err(TinkerforgeError::NoResponseReceived)
                }
            } else {
                self.send_packet(&request, seq, false).await?;
                Ok(None)
            }
        }

        fn filter_response(uid: Uid, function_id: u8, seq: u8) -> impl Fn(&Result<PacketData, BroadcastStreamRecvError>) -> bool {
            move |result| {
                if let Ok(PacketData { header, .. }) = result {
                    header.uid == uid && header.function_id == function_id && header.sequence_number == seq
                } else {
                    false
                }
            }
        }
        pub async fn get(&mut self, uid: Uid, function_id: u8, payload: &[u8], timeout: Duration) -> Result<PacketData, TinkerforgeError> {
            let request = Request::Get { uid, function_id, payload };
            let seq = self.next_seq();
            let stream = BroadcastStream::new(self.receiver.resubscribe())
                .map_while(Self::while_some)
                .filter(Self::filter_response(uid, function_id, seq))
                .timeout(timeout);
            tokio::pin!(stream);
            self.send_packet(&request, seq, true).await?;
            Ok(stream.next().await.ok_or(TinkerforgeError::NoResponseReceived)?.map_err(|_| TinkerforgeError::NoResponseReceived)??)
        }

        fn while_some(v: Result<Option<PacketData>, BroadcastStreamRecvError>) -> Option<Result<PacketData, BroadcastStreamRecvError>> {
            match v {
                Ok(None) => None,
                Ok(Some(p)) => Some(Ok(p)),
                Err(e) => Some(Err(e)),
            }
        }
        pub(crate) async fn callback_stream(&mut self, uid: Uid, function_id: u8) -> impl Stream<Item = PacketData> {
            BroadcastStream::new(self.receiver.resubscribe())
                .map_while(move |result| match result {
                    Ok(Some(p)) => {
                        let header = &p.header;

                        if header.uid == uid && header.function_id == function_id {
                            Some(Some(p))
                        } else if header.function_id == 253 {
                            let enum_paket = EnumerateResponse::from_le_byte_slice(p.body());
                            if enum_paket.enumeration_type == EnumerationType::Disconnected && Some(uid) == enum_paket.uid.parse().ok() {
                                // device is disconnected -> end stream
                                None
                            } else {
                                Some(None)
                            }
                        } else {
                            Some(None)
                        }
                    }
                    Ok(None) => None,
                    Err(BroadcastStreamRecvError::Lagged(count)) => {
                        warn!("Slow receiver, skipped {count} Packets");
                        Some(None)
                    }
                })
                .filter_map(|f| f)
        }
        async fn send_packet(&mut self, request: &Request<'_>, seq: u8, response_expected: bool) -> Result<(), TinkerforgeError> {
            let header = request.get_header(response_expected, seq);
            assert!(header.length <= 72);
            let mut result = vec![0; header.length as usize];
            header.uid.write_to_slice(&mut result[0..4]);
            result[4] = header.length;
            result[5] = header.function_id;
            result[6] = header.sequence_number << 4 | (header.response_expected as u8) << 3;
            result[7] = header.error_code << 6;
            let payload = request.get_payload();
            if !payload.is_empty() {
                result[8..].copy_from_slice(payload);
            }
            self.write_stream.write_all(&result[..]).await?;
            debug!("Sent: {request:?}");
            Ok(())
        }
        fn next_seq(&mut self) -> u8 {
            self.seq_num += 1;
            if self.seq_num > 15 {
                self.seq_num = 1;
            }
            self.seq_num
        }
    }

    impl Drop for InnerAsyncIpConnection {
        fn drop(&mut self) {
            self.abort_handle.abort();
        }
    }

    #[derive(Clone, Debug)]
    pub(crate) struct PacketData {
        header: PacketHeader,
        body: Box<[u8]>,
    }

    impl PacketData {
        #[allow(dead_code)]
        pub fn header(&self) -> PacketHeader {
            self.header
        }
        pub fn body(&self) -> &[u8] {
            &self.body
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) enum Request<'a> {
        Set { uid: Uid, function_id: u8, payload: &'a [u8] },
        Get { uid: Uid, function_id: u8, payload: &'a [u8] },
    }

    impl Request<'_> {
        fn get_header(&self, response_expected: bool, sequence_number: u8) -> PacketHeader {
            match self {
                Request::Set { uid, function_id, payload } => {
                    PacketHeader::with_payload(*uid, *function_id, sequence_number, response_expected, payload.len() as u8)
                }
                Request::Get { uid, function_id, payload, .. } => {
                    PacketHeader::with_payload(*uid, *function_id, sequence_number, true, payload.len() as u8)
                }
            }
        }
        fn get_payload(&self) -> &[u8] {
            match self {
                Request::Set { payload, .. } => payload,
                Request::Get { payload, .. } => payload,
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct PacketHeader {
    uid: Uid,
    length: u8,
    function_id: u8,
    sequence_number: u8,
    response_expected: bool,
    error_code: u8,
}

impl PacketHeader {
    pub(crate) fn with_payload(uid: Uid, function_id: u8, sequence_number: u8, response_expected: bool, payload_len: u8) -> PacketHeader {
        PacketHeader { uid, length: PacketHeader::SIZE as u8 + payload_len, function_id, sequence_number, response_expected, error_code: 0 }
    }

    pub(crate) const SIZE: usize = 8;
}

impl FromByteSlice for PacketHeader {
    fn from_le_byte_slice(bytes: &[u8]) -> PacketHeader {
        PacketHeader {
            uid: Uid::from_le_byte_slice(bytes),
            length: bytes[4],
            function_id: bytes[5],
            sequence_number: (bytes[6] & 0xf0) >> 4,
            response_expected: (bytes[6] & 0x08) != 0,
            error_code: (bytes[7] & 0xc0) >> 6,
        }
    }

    fn bytes_expected() -> usize {
        8
    }
}

impl ToBytes for PacketHeader {
    fn write_to_slice(self, target: &mut [u8]) {
        self.uid.write_to_slice(&mut target[0..4]);
        target[4] = self.length;
        target[5] = self.function_id;
        target[6] = self.sequence_number << 4 | (self.response_expected as u8) << 3;
        target[7] = self.error_code << 6;
    }
}

//const MAX_PACKET_SIZE: usize = PacketHeader::SIZE + 64 + 8; //header + payload + optional data

/// Type of enumeration of a device.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EnumerationType {
    /// Device is available (enumeration triggered by user: [`Enumerate`](crate::ip_connection::IpConnection::enumerate())).
    /// This enumeration type can occur multiple times for the same device.
    Available,
    /// Device is newly connected (automatically send by Brick after establishing a communication connection).
    /// This indicates that the device has potentially lost its previous configuration and needs to be reconfigured.
    Connected,
    /// Device is disconnected (only possible for USB connection). In this case only uid and enumerationType are valid.
    Disconnected,
    /// Device returned an unknown enumeration type.
    Unknown,
}

impl From<u8> for EnumerationType {
    fn from(byte: u8) -> EnumerationType {
        match byte {
            0 => EnumerationType::Available,
            1 => EnumerationType::Connected,
            2 => EnumerationType::Disconnected,
            _ => EnumerationType::Unknown,
        }
    }
}

/// Devices send `EnumerateResponse`s when they are connected, disconnected or when an enumeration was
/// triggered by the user using the [`Enumerate`](crate::ip_connection::IpConnection::enumerate) method.
#[derive(Clone, Debug)]
pub struct EnumerateResponse {
    /// The UID of the device.
    pub uid: String,
    /// UID where the device is connected to.
    /// For a Bricklet this is the UID of the Brick or Bricklet it is connected to.
    /// For a Brick it is the UID of the bottommost Brick in the stack.
    /// For the bottommost Brick in a stack it is "0".
    /// With this information it is possible to reconstruct the complete network topology.
    pub connected_uid: String,
    /// For Bricks: '0' - '8' (position in stack). For Bricklets: 'a' - 'd' (position on Brick).
    pub position: char,
    /// Major, minor and release number for hardware version.
    pub hardware_version: [u8; 3],
    /// Major, minor and release number for firmware version.
    pub firmware_version: [u8; 3],
    /// A number that represents the device.
    /// The device identifier numbers can be found [here](https://www.tinkerforge.com/en/doc/Software/Device_Identifier.html).
    /// There are also constants for these numbers named following this pattern:
    ///
    /// <device-class>.DEVICE_IDENTIFIER
    ///
    /// For example: MasterBrick.DEVICE_IDENTIFIER or AmbientLightBricklet.DEVICE_IDENTIFIER.
    pub device_identifier: ParsedOrRaw<DeviceIdentifier, u16>,
    /// Type of enumeration. See [`EnumerationType`](crate::ip_connection::EnumerationType)
    pub enumeration_type: EnumerationType,
}

impl EnumerateResponse {
    pub fn uid_as_number(&self) {}
}

impl FromByteSlice for EnumerateResponse {
    fn from_le_byte_slice(bytes: &[u8]) -> EnumerateResponse {
        EnumerateResponse {
            uid: str::from_utf8(&bytes[0..8])
                .expect("Could not convert to string. This is a bug in the rust bindings.")
                .replace('\u{0}', ""),
            connected_uid: str::from_utf8(&bytes[8..16])
                .expect("Could not convert to string. This is a bug in the rust bindings.")
                .replace('\u{0}', ""),
            position: bytes[16] as char,
            hardware_version: [bytes[17], bytes[18], bytes[19]],
            firmware_version: [bytes[20], bytes[21], bytes[22]],
            device_identifier: ParsedOrRaw::<DeviceIdentifier, u16>::from_le_byte_slice(&bytes[23..25]),
            enumeration_type: EnumerationType::from(bytes[25]),
        }
    }

    fn bytes_expected() -> usize {
        26
    }
}

struct ServerNonce([u8; 4]);

impl FromByteSlice for ServerNonce {
    fn from_le_byte_slice(bytes: &[u8]) -> ServerNonce {
        ServerNonce([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn bytes_expected() -> usize {
        4
    }
}

/// This error is returned if the remote's server nonce could not be queried.
#[derive(Debug, Copy, Clone)]
pub enum AuthenticateError {
    SecretInvalid,
    CouldNotGetServerNonce,
}

impl std::fmt::Display for AuthenticateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AuthenticateError::SecretInvalid => {
                    "Authentication secret contained non-ASCII characters"
                }
                AuthenticateError::CouldNotGetServerNonce => "Could not get server nonce",
            }
        )
    }
}

impl std::error::Error for AuthenticateError {}
