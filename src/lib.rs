extern crate jsonrpc;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate strason;

use std::fmt::{self, Display, Formatter};

use jsonrpc::client::Client;
use strason::Json;

pub mod net;
pub mod mining;

macro_rules! rpc_request {
    ($client:expr, $name:expr, $params:expr) => {
        {
            let request = $client.build_request($name, $params);
            $client.send_request(&request)
                .map_err(|e| $crate::Error::new(e.into(), "RPC error"))?
        }
    }
}

macro_rules! rpc_method {
    (
        $(#[$outer:meta])*
        pub fn $rpc_method:ident(&self) -> RpcResult<$ty:ty>
    ) => {
        $(#[$outer:meta])*
        pub fn $rpc_method(&self) -> $crate::RpcResult<$ty> {
            let response = rpc_request!(&self.client,
                                        stringify!($rpc_method).to_string(),
                                        vec![]);

            let v: $ty = response.into_result()
                .map_err(|e| $crate::Error::new(e.into(), "Malformed response"))?;

            Ok(v)
        }
    }
}

pub type RpcResult<T> = Result<T, Error>;

/// A Handle to a Bitcoin JSON-RPC connection
pub struct BitcoinRpc {
    client: Client,
}

impl BitcoinRpc {
    /// Creates a client to a bitcoind JSON-RPC server.
    pub fn new(url: String, user: Option<String>, pass: Option<String>) -> Self {
        // Check that if we have a password, we have a username; other way
        // around is ok.
        debug_assert!(pass.is_none() || user.is_some());

        BitcoinRpc { client: Client::new(url, user, pass) }
    }

    pub fn estimatesmartfee<E>(
        &self,
        conf_target: u16,
        estimate_mode: E,
    ) -> Result<mining::EstimateSmartFee, Error>
    where E:
          Into<Option<mining::EstimateMode>>
    {
        let mut params = Vec::new();
        params.push(Json::from_serialize(conf_target).unwrap());
        if let Some(estimate_mode) = estimate_mode.into() {
            params.push(Json::from_serialize(estimate_mode).unwrap())
        }

        let response = rpc_request!(&self.client,
                                    "estimatesmartfee".to_string(),
                                    params);

        let v: mining::EstimateSmartFee = response.into_result()
            .map_err(|e| Error::new(e.into(), "Malformed response"))?;

        Ok(v)
    }

    rpc_method!(pub fn getconnectioncount(&self) -> RpcResult<u64>);
    rpc_method!(pub fn ping(&self) -> RpcResult<()>);
    rpc_method!(pub fn getnetworkinfo(&self) -> RpcResult<net::NetworkInfo>);
}

/// The error type for bitcoin JSON-RPC operations.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    desc: String,
}

impl Error {
    fn new<D>(kind: ErrorKind, desc: D) -> Error
    where
        D: ToString,
    {
        Error {
            kind,
            desc: desc.to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::JsonRpc(ref e) => {
                write!(fmt, "JSON-RPC error: {} ({})", self.desc, e)
            },
            ErrorKind::Daemon => write!(fmt, "bitcoind daemon error: {}", self.desc),
            ErrorKind::Other => write!(fmt, "{}", self.desc),
        }
    }
}

/// The kind of error.
#[derive(Debug)]
pub enum ErrorKind {
    /// A JSON-RPC error.
    JsonRpc(jsonrpc::Error),
    /// The daemon failed to give a valid response.
    Daemon,
    /// Any other error.
    Other,
}

impl From<jsonrpc::Error> for ErrorKind {
    fn from(e: jsonrpc::Error) -> ErrorKind {
        ErrorKind::JsonRpc(e)
    }
}
