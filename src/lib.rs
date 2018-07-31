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

    pub fn estimatesmartfee<E>(
        &self,
        conf_target: u16,
        estimate_mode: E,
    ) -> Result<EstimateSmartFee, Error>
    where E:
          Into<Option<EstimateMode>>
    {
        let mut params = Vec::new();
        params.push(Json::from_serialize(conf_target).unwrap());
        if let Some(estimate_mode) = estimate_mode.into() {
            params.push(Json::from_serialize(estimate_mode).unwrap())
        }
        let request = self.client.build_request("estimatesmartfee".to_string(),
                                                params);
        let response = self.client.send_request(&request)
            .map_err(|e| Error::new(e.into(), "RPC error"))?;

        let v: EstimateSmartFee = response.into_result()
            .map_err(|e| Error::new(e.into(), "Malformed response"))?;

        Ok(v)
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
    /// The server version
    pub version: i64,
    /// The server subversion string
    pub subversion: String,
    /// The protocol version
    pub protocolversion: i64,
    /// The services we offer to the network
    pub localservices: Option<String>,
    /// `true` if transaction relay is requested from peers
    pub localrelay: bool,
    /// The time offset
    pub timeoffset: i64,
    /// Wheter p2p networking is enabled
    pub networkactive: Option<bool>,
    /// The number of connections
    pub connections: Option<i64>,
    /// Information per network
    pub networks: Vec<Network>,
    /// Minimum relay fee for transactions in BTC/kB
    pub relayfee: Json,
    /// Minimum fee increment for mempool limiting or BIP 125 replacement in
    /// BTC/kB
    pub incrementalfee: Json,
    /// List of local addresses
    pub localaddresses: Vec<LocalAddress>,
    /// Any network and blockchain warnings
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

/// Network information
#[derive(Debug, Deserialize, Serialize)]
pub struct Network {
    /// The name
    pub name: NetworkName,
    /// Is the network limited using `-onlynet`?
    pub limited: bool,
    /// Is the network reachable?
    pub reachable: bool,
    /// The proxy that is used for this network
    pub proxy: String,
    /// Whether randomized credentials are used
    pub proxy_randomize_credentials: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalAddress {
    /// Network address
    pub address: String,
    /// Network port
    pub port: u16,
    /// Relative score
    pub score: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EstimateSmartFee {
    /// Estimate fee rate in BTC/kB.
    pub feerate: Json,
    /// Errors encountered during processing.
    pub errors: Vec<String>,
    /// Block number where estimate was found.
    pub blocks: i64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum EstimateMode {
    Unset,
    Economical,
    Conservative,
}

impl FromStr for EstimateMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UNSET" => Ok(EstimateMode::Unset),
            "ECONOMICAL" => Ok(EstimateMode::Economical),
            "CONSERVATIVE" => Ok(EstimateMode::Conservative),
            _ => Err(Error::new(ErrorKind::Other, "invalid network name")),
        }
    }
}

impl<'de> serde::de::Deserialize<'de> for EstimateMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = EstimateMode;

            fn expecting(&self, fmt: &mut Formatter) -> fmt::Result {
                write!(fmt, "estimate mode")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                EstimateMode::from_str(v)
                    .map_err(serde::de::Error::custom)
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                EstimateMode::from_str(v)
                    .map_err(serde::de::Error::custom)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                EstimateMode::from_str(&*v)
                    .map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl serde::ser::Serialize for EstimateMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let s = match *self {
            EstimateMode::Unset => "UNSET",
            EstimateMode::Economical => "ECONOMICAL",
            EstimateMode::Conservative => "CONSERVATIVE",
        };

        serializer.serialize_str(s)
    }
}
