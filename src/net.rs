//! Network related RPC result types.

use std::fmt::{self, Formatter};
use std::str::FromStr;

use serde::{de, ser};
use strason::Json;

use {Error, ErrorKind};

/// The result of "getnetworkinfo"
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

impl<'de> de::Deserialize<'de> for NetworkName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = NetworkName;

            fn expecting(&self, fmt: &mut Formatter) -> fmt::Result {
                write!(fmt, "network name")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                NetworkName::from_str(v)
                    .map_err(de::Error::custom)
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                NetworkName::from_str(v)
                    .map_err(de::Error::custom)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                NetworkName::from_str(&*v)
                    .map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl ser::Serialize for NetworkName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
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
