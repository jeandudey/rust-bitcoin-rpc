// Copyright 2018 Jean Pierre Dudey <jeandudey@hotmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Network related RPC result types.

use std::fmt::{self, Formatter};
use std::str::FromStr;

use serde::{de, ser};
use strason::Json;

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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ipv4" => Ok(NetworkName::Ipv4),
            "ipv6" => Ok(NetworkName::Ipv6),
            "onion" => Ok(NetworkName::Onion),
            _ => Err(()),
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
                    .map_err(|_e| de::Error::custom("invalid string"))
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                NetworkName::from_str(v)
                    .map_err(|_e| de::Error::custom("invalid string"))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                NetworkName::from_str(&*v)
                    .map_err(|_e| de::Error::custom("invalid string"))
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

/// Models the result of "getpeerinfo"
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PeerInfo {
    /// Peer index
    pub id: u64,
    /// The IP address and port of the peer
    // TODO: use a type for addr
    pub addr: String,
    /// Bind address of the connection to the peer
    // TODO: use a type for addrbind
    pub addrbind: String,
    /// Local address as reported by the peer
    // TODO: use a type for addrlocal
    pub addrlocal: String,
    /// The services offered
    // TODO: use a type for services
    pub services: String,
    /// Whether peer has asked us to relay transactions to it
    pub relaytxes: bool,
    /// The time in seconds since epoch (Jan 1 1970 GMT) of the last send
    pub lastsend: u64,
    /// The time in seconds since epoch (Jan 1 1970 GMT) of the last receive
    pub lastrecv: u64,
    /// The total bytes sent
    pub bytessent: u64,
    /// The total bytes received
    pub bytesrecv: u64,
    /// The connection time in seconds since epoch (Jan 1 1970 GMT)
    pub conntime: u64,
    /// The time offset in seconds
    pub timeoffset: u64,
    /// ping time (if available)
    pub pingtime: u64,
    /// minimum observed ping time (if any at all)
    pub minping: u64,
    /// ping wait (if non-zero)
    pub pingwait: u64,
    /// The peer version, such as 70001
    pub version: u64,
    /// The string version
    pub subver: String,
    /// Inbound (true) or Outbound (false)
    pub inbound: bool,
    /// Whether connection was due to `addnode`/`-connect` or if it was an
    /// automatic/inbound connection
    pub addnode: bool,
    /// The starting height (block) of the peer
    pub startingheight: u64,
    /// The ban score
    pub banscore: i64,
    /// The last header we have in common with this peer
    pub synced_headers: u64,
    /// The last block we have in common with this peer
    pub synced_blocks: u64,
    /// The heights of blocks we're currently asking from this peer
    pub inflight: Vec<u64>,
    /// Whether the peer is whitelisted
    pub whitelisted: bool,
    /// The total bytes sent aggregated by message type
    // TODO: use a type for bytessent_per_msg
    pub bytessent_per_msg: Json,
    /// The total bytes received aggregated by message type
    // TODO: use a type for bytesrecv_per_msg
    pub bytesrecv_per_msg: Json,
}

/// "addnode" command.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AddNode {
    /// Add the node
    Add,
    /// Remove the node
    Remove,
    /// Try to connect once to the node
    OneTry,
}

impl FromStr for AddNode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "add" => Ok(AddNode::Add),
            "remove" => Ok(AddNode::Remove),
            "onetry" => Ok(AddNode::OneTry),
            _ => Err(()),
        }
    }
}

impl<'de> de::Deserialize<'de> for AddNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = AddNode;

            fn expecting(&self, fmt: &mut Formatter) -> fmt::Result {
                write!(fmt, "network name")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                AddNode::from_str(v)
                    .map_err(|_e| de::Error::custom("invalid string"))
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                AddNode::from_str(v)
                    .map_err(|_e| de::Error::custom("invalid string"))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                AddNode::from_str(&*v)
                    .map_err(|_e| de::Error::custom("invalid string"))
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl ser::Serialize for AddNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let s = match *self {
            AddNode::Add => "add",
            AddNode::Remove => "remove",
            AddNode::OneTry => "onetry",
        };

        serializer.serialize_str(s)
    }
}
