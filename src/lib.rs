// Copyright 2018 Jean Pierre Dudey <jeandudey@hotmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate jsonrpc;

extern crate serde;
extern crate strason;

extern crate bitcoin;
extern crate bitcoin_rpc_json;

extern crate failure;
#[macro_use]
extern crate failure_derive;

/// Blockchain related RPC result types.
pub mod blockchain {
    #[doc(inline)]
    pub use bitcoin_rpc_json::blockchain::*;
}

/// Mining related RPC result types.
pub mod mining {
    #[doc(inline)]
    pub use bitcoin_rpc_json::mining::*;
}

/// Network related RPC result types.
pub mod net {
    #[doc(inline)]
    pub use bitcoin_rpc_json::net::*;
}

use jsonrpc::client::Client;
use strason::Json;

use failure::ResultExt;

use bitcoin::util::hash::Sha256dHash;

macro_rules! rpc_request {
    ($client:expr, $name:expr, $params:expr) => {
        {
            let request = $client.build_request($name, $params);
            let response = $client.send_request(&request)
                .context(ErrorKind::BadResponse)?;
            response.into_result()
                .context(ErrorKind::MalformedResponse)?
        }
    }
}

macro_rules! rpc_method {
    (
        $(#[$outer:meta])*
        pub fn $rpc_method:ident(&self) -> RpcResult<$ty:ty>;
    ) => {
        $(#[$outer])*
        pub fn $rpc_method(&self) -> $crate::RpcResult<$ty> {
            let v: $ty = rpc_request!(&self.client,
                                      stringify!($rpc_method).to_string(),
                                      vec![]);
            Ok(v)
        }
    };
    (
        $(#[$outer:meta])*
        pub fn $rpc_method:ident(&self, $($param:ident : $pty:ty),+) -> RpcResult<$ty:ty>;
    ) => {
        $(#[$outer])*
        pub fn $rpc_method(&self, $($param: $pty),+) -> $crate::RpcResult<$ty> {
            let mut params = Vec::new();
            $(
                params.push(Json::from_serialize(&$param).unwrap());
            )+

            let v: $ty = rpc_request!(&self.client,
                                      stringify!($rpc_method).to_string(),
                                      params);
            Ok(v)
        }
    };
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

    // blockchain

    rpc_method! {
        /// Returns the numbers of block in the longest chain.
        pub fn getblockcount(&self) -> RpcResult<u64>;
    }

    /// Returns the hash of the best (tip) block in the longest blockchain.
    pub fn getbestblockhash(&self) -> RpcResult<Sha256dHash> {
        let v: String = rpc_request!(&self.client,
                                     "getbestblockhash".to_string(),
                                      vec![]);
        Ok(Sha256dHash::from_hex(&v).unwrap())
    }

    /// Waits for a specific new block and returns useful info about it.
    /// Returns the current block on timeout or exit.
    ///
    /// # Arguments
    ///
    /// 1. `timeout`: Time in milliseconds to wait for a response. 0
    /// indicates no timeout.
    pub fn waitfornewblock(
        &self,
        timeout: u64,
    ) -> RpcResult<blockchain::BlockRef> {
        let params = vec![Json::from_serialize(timeout).unwrap()];

        let v: blockchain::SerdeBlockRef = rpc_request!(&self.client,
                                                        "waitfornewblock".to_string(),
                                                        params);
        Ok(v.into())
    }

    /// Waits for a specific new block and returns useful info about it.
    /// Returns the current block on timeout or exit.
    ///
    /// # Arguments
    ///
    /// 1. `blockhash`: Block hash to wait for.
    /// 2. `timeout`: Time in milliseconds to wait for a response. 0
    /// indicates no timeout.
    pub fn waitforblock(
        &self,
        blockhash: String,
        timeout: u64,
    ) -> RpcResult<blockchain::BlockRef> {
        let params = vec![Json::from_serialize(blockhash).unwrap(),
                          Json::from_serialize(timeout).unwrap()];

        let v: blockchain::SerdeBlockRef = rpc_request!(&self.client,
                                                        "waitforblock".to_string(),
                                                        params);
        Ok(v.into())
    }

    rpc_method! {
        /// Returns a data structure containing various state info regarding
        /// blockchain processing.
        pub fn getblockchaininfo(&self) -> RpcResult<blockchain::BlockchainInfo>;
    }

    // mining

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
        Ok(response)
    }

    // net
    
    rpc_method! {
        /// Returns the number of connections to other nodes.
        pub fn getconnectioncount(&self) -> RpcResult<u64>;
    }

    rpc_method! {
        /// Requests that a ping be sent to all other nodes, to measure ping
        /// time.
        ///
        /// Results provided in `getpeerinfo`, `pingtime` and `pingwait` fields
        /// are decimal seconds.
        ///
        /// Ping command is handled in queue with all other commands, so it
        /// measures processing backlog, not just network ping.
        pub fn ping(&self) -> RpcResult<()>;
    }

    rpc_method! {
        /// Returns data about each connected network node as an array of
        /// [`PeerInfo`][]
        ///
        /// [`PeerInfo`]: net/struct.PeerInfo.html
        pub fn getpeerinfo(&self) -> RpcResult<Vec<net::PeerInfo>>;
    }

    rpc_method! {
        /// Attempts to add or remove a node from the addnode list.
        ///
        /// Or try a connection to a node once.
        ///
        /// Nodes added using `addnode` (or `-connect`) are protected from DoS
        /// disconnection and are not required to be full nodes/support SegWit
        /// as other outbound peers are (though such peers will not be synced
        /// from).
        ///
        /// # Arguments:
        ///
        /// 1. `node`: The node (see [`getpeerinfo`][] for nodes)
        /// 2. `command`: `AddNode::Add` to add a node to the list,
        /// `AddNode::Remove` to remove a node from the list, `AddNode::OneTry`
        /// to try a connection to the node once
        ///
        /// [`getpeerinfo`]: #method.getpeerinfo
        pub fn addnode(
            &self,
            node: &str,
            commnad: net::AddNode
        ) -> RpcResult<()>;
    }

    rpc_method! {
        pub fn getnetworkinfo(&self) -> RpcResult<net::NetworkInfo>;
    }
}

/// The error type for bitcoin JSON-RPC operations.
#[derive(Debug)]
pub struct Error {
    kind: failure::Context<ErrorKind>,
}

impl From<ErrorKind> for Error {
    fn from(e: ErrorKind) -> Error {
        Error {
            kind: failure::Context::new(e),
        }
    }
}

impl From<failure::Context<ErrorKind>> for Error {
    fn from(e: failure::Context<ErrorKind>) -> Error {
        Error {
            kind: e,
        }
    }
}

/// The kind of error.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Fail)]
pub enum ErrorKind {
    /// The request resulted in an error.
    #[fail(display = "Request resulted in an error")]
    BadResponse,
    /// The received response format is malformed.
    #[fail(display = "Response format is invalid")]
    MalformedResponse,
}
