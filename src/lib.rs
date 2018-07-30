extern crate jsonrpc;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate strason;

extern crate bitcoin_amount;

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use jsonrpc::client::Client;
use strason::Json;

use bitcoin_amount::Amount;

macro_rules! rpc_method {
    (
        $(#[$outer:meta])*
        pub fn $rpc_method:ident(&self) -> RpcResult<$ty:ty>
    ) => {
        $(#[$outer:meta])*
        pub fn $rpc_method(&self) -> $crate::RpcResult<$ty> {
            let request = self.client.build_request(stringify!($rpc_method).to_string(),
                                                    vec![]);

            let response = self.client.send_request(&request)
                .map_err(|e| $crate::Error::new(e.into(), "RPC error"))?;

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

    /// Get the estimated fee per kB for a transaction.
    ///
    /// The parameter specifies how many blocks a transaction may wait to
    /// be included in a block. It SHOULD be between 2 and 25.
    ///
    /// # Panics
    ///
    /// If the `blocks` parameter isn't between 2 and 25.
    pub fn estimatefee(&self, blocks: u8) -> Result<Amount, Error> {
        assert!(blocks >= 2 && blocks <= 25, "`blocks` is out of range");

        let request = self.client.build_request("estimatefee".to_string(),
                                                vec![]);
        let response = self.client.send_request(&request)
            .map_err(|e| Error::new(e.into(), "RPC error"))?;

        if let Some(e) = response.error {
            let kind = jsonrpc::Error::Rpc(e).into();
            return Err(Error::new(kind, "JSON-RPC error"));
        }

        let res = match response.result {
            Some(res) => res,
            None => {
                let kind = jsonrpc::Error::NoErrorOrResult.into();
                return Err(Error::new(kind, "JSON-RPC response error"))
            },
        };

        if let Some(btc) = res.num() {
            let amt = Amount::from_str(btc)
                .map_err(|e| Error::new(e.into(), "fee isn't a valid number"))?;

            if amt < Amount::zero() {
                return Err(Error::new(ErrorKind::Daemon,
                                      "invalid fee"));
            }

            return Ok(amt)
        }

        Err(Error::new(ErrorKind::Daemon, "fee is not a number"))
    }

    rpc_method!(pub fn getnetworkinfo(&self) -> RpcResult<NetworkInfo>);
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
            ErrorKind::ParseAmount(ref e) => {
                write!(fmt, "BTC amount error: {} ({})", self.desc, e)
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
    /// A JSON-RPC error.
    ParseAmount(bitcoin_amount::ParseAmountError),
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

impl From<bitcoin_amount::ParseAmountError> for ErrorKind {
    fn from(e: bitcoin_amount::ParseAmountError) -> ErrorKind {
        ErrorKind::ParseAmount(e)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkInfo {
    pub version: i64,
    pub subversion: String,
    pub protocolversion: i64,
    // XXX: Add an special type for this?
    pub localservices: String,
    pub localrelay: bool,
    pub timeoffset: i64,
    pub networkactive: bool,
    pub connections: i64,
    pub networks: Option<Vec<Network>>,
    pub relayfee: Json,
    pub incrementalfee: Json,
    pub localaddresses: Json,
    pub warnings: String,
}

/// Network name.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum NetworkName {
    /// IPv4
    Ipv4,
    /// IPv6
    Ipv6,
    /// Onion
    Onion,
}

impl FromStr for NetworkName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ipv4" => Ok(NetworkName::Ipv4),
            "ipv6" => Ok(NetworkName::Ipv6),
            "onion" => Ok(NetworkName::Onion),
            _ => Err(Error::new(ErrorKind::Other, "invalid network name")),
        }
    }
}

impl<'de> serde::de::Deserialize<'de> for NetworkName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = NetworkName;

            fn expecting(&self, fmt: &mut Formatter) -> fmt::Result {
                write!(fmt, "network name")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                NetworkName::from_str(v)
                    .map_err(serde::de::Error::custom)
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                NetworkName::from_str(v)
                    .map_err(serde::de::Error::custom)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                NetworkName::from_str(&*v)
                    .map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl serde::ser::Serialize for NetworkName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let s = match *self {
            NetworkName::Ipv4 => "ipv4",
            NetworkName::Ipv6 => "ipv6",
            NetworkName::Onion => "onion",
        };

        serializer.serialize_str(s)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Network {
    pub name: NetworkName,
    pub limited: bool,
    pub reachable: bool,
    pub proxy: String,
    pub proxy_randomize_credentials: bool,
}
