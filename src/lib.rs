// Copyright 2018 Jean Pierre Dudey <jeandudey@hotmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate jsonrpc;

extern crate serde;
extern crate serde_json;

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

use bitcoin::util::hash::Sha256dHash;

fn sha256dhash_from_str(rpc_name: &'static str, hex: &str) -> RpcResult<Sha256dHash> {
    Ok(Sha256dHash::from_hex(&hex).map_err(|_e| Error::MalformedResponse { rpc_name })?)
}

macro_rules! rpc_method {
    (
        $(#[$outer:meta])*
        pub fn $rpc_method:ident(&self) -> RpcResult<$ty:ty>;
    ) => {
        $(#[$outer])*
        pub fn $rpc_method(&self) -> $crate::RpcResult<$ty> {
            let v: $ty = self.do_rpc(stringify!($rpc_method), vec![])?;
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
                params.push(serde_json::to_value(&$param).unwrap());
            )+

            let v: $ty = self.do_rpc(stringify!($rpc_method), params)?;
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

        BitcoinRpc {
            client: Client::new(url, user, pass),
        }
    }

    pub fn do_rpc<T: for<'a> serde::de::Deserialize<'a>>(
        &self,
        rpc_name: &'static str,
        args: Vec<serde_json::value::Value>,
    ) -> RpcResult<T> {
        let request = self.client.build_request(rpc_name.to_string(), args);
        let response = self
            .client
            .send_request(&request)
            .map_err(|e| (rpc_name, e))?;

        Ok(response.clone().into_result().map_err(|e| (rpc_name, e))?)
    }

    // blockchain

    rpc_method! {
        /// Returns the numbers of block in the longest chain.
        pub fn getblockcount(&self) -> RpcResult<u64>;
    }

    /// Returns the hash of the best (tip) block in the longest blockchain.
    pub fn getbestblockhash(&self) -> RpcResult<Sha256dHash> {
        let v: String = self.do_rpc("getbestblockhash", vec![])?;
        sha256dhash_from_str("getbestblockhash", &v)
    }

    /// Waits for a specific new block and returns useful info about it.
    /// Returns the current block on timeout or exit.
    ///
    /// # Arguments
    ///
    /// 1. `timeout`: Time in milliseconds to wait for a response. 0
    /// indicates no timeout.
    pub fn waitfornewblock(&self, timeout: u64) -> RpcResult<blockchain::BlockRef> {
        let params = vec![serde_json::to_value(timeout).unwrap()];

        let v: blockchain::SerdeBlockRef = self.do_rpc("waitfornewblock", params)?;
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
    pub fn waitforblock(&self, blockhash: String, timeout: u64) -> RpcResult<blockchain::BlockRef> {
        let params = vec![
            serde_json::to_value(blockhash).unwrap(),
            serde_json::to_value(timeout).unwrap(),
        ];

        let v: blockchain::SerdeBlockRef = self.do_rpc("waitforblock", params)?;
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
    where
        E: Into<Option<mining::EstimateMode>>,
    {
        let mut params = Vec::new();
        params.push(serde_json::to_value(conf_target).unwrap());
        if let Some(estimate_mode) = estimate_mode.into() {
            params.push(serde_json::to_value(estimate_mode).unwrap())
        }

        let response = self.do_rpc("estimatesmartfee", params)?;
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

    /// Mark a block as invalid by `block_hash`
    pub fn invalidate_block(&self, block_hash: &Sha256dHash) -> RpcResult<()> {
        self.do_rpc("invalidateblock", vec![block_hash.to_string().into()])
    }

    /// Get the hex-consensus-encoded block by `block_hash`
    pub fn get_block(&self, block_hash: &Sha256dHash) -> RpcResult<String> {
        self.do_rpc("getblock", vec![block_hash.to_string().into(), 0.into()])
    }

    /// Get block by `block_hash`
    pub fn get_block_verbose(&self, block_hash: &Sha256dHash) -> RpcResult<blockchain::BlockInfo> {
        self.do_rpc("getblock", vec![block_hash.to_string().into(), 1.into()])
    }

    /// Generate new address under own control
    pub fn get_new_address(&self, account: String) -> RpcResult<String> {
        self.do_rpc("getnewaddress", vec![account.into()])
    }

    /// Dump private key of an `address`
    pub fn dump_priv_key(&self, address: String) -> RpcResult<String> {
        self.do_rpc("dumpprivkey", vec![address.into()])
    }

    /// Mine `block_num` blocks and pay coinbase to `address`
    ///
    /// Returns hashes of the generated blocks
    pub fn generate_to_address(
        &self,
        block_num: u64,
        address: String,
    ) -> RpcResult<Vec<Sha256dHash>> {
        let v: Vec<String> =
            self.do_rpc("generatetoaddress", vec![block_num.into(), address.into()])?;

        Ok(v.into_iter()
            .map(|v| sha256dhash_from_str("generatetoaddress", &v))
            .collect::<RpcResult<Vec<Sha256dHash>>>()?)
    }

    /// Get block hash at a given height
    pub fn get_blockhash(&self, height: u64) -> RpcResult<Sha256dHash> {
        let hex_string: String = self.do_rpc("getblockhash", vec![height.into()])?;
        sha256dhash_from_str("getblockhash", &hex_string)
    }

    pub fn create_raw_transaction(
        &self,
        ins: &[self::blockchain::TxInInfoCreateTx],
        outs: &std::collections::HashMap<AddressString, BalanceFloat>,
    ) -> RpcResult<RawTxString> {
        self.do_rpc(
            "createrawtransaction",
            vec![
                serde_json::to_value(ins).unwrap(),
                serde_json::to_value(outs).unwrap(),
            ],
        )
    }

    pub fn sign_raw_transaction(
        &self,
        unsigned: RawTxString,
        ins: &[self::blockchain::TxInInfoSignTx],
        privkeys: &[PrivkeyString],
    ) -> RpcResult<self::blockchain::SignedRawTransaction> {
        self.do_rpc(
            "signrawtransaction",
            vec![
                unsigned.into(),
                serde_json::to_value(ins).unwrap(),
                serde_json::to_value(privkeys).unwrap(),
            ],
        )
    }

    pub fn send_raw_transaction(&mut self, tx: RawTransactionString) -> RpcResult<RawTxString> {
        self.do_rpc("sendrawtransaction", vec![tx.into()])
    }

    /// Get the hex-consensus-encoded transaction by `txid`
    pub fn get_raw_transaction(&self, hash: &Sha256dHash) -> RpcResult<String> {
        self.do_rpc("getrawtransaction", vec![hash.to_string().into(), 0.into()])
    }
}

pub type RawTransactionString = String;
pub type AddressString = String;
pub type PrivkeyString = String;
pub type BalanceFloat = f64;
pub type RawTxString = String;

impl From<(&'static str, jsonrpc::Error)> for Error {
    fn from(e: (&'static str, jsonrpc::Error)) -> Error {
        Error::JsonRpc {
            rpc_name: e.0,
            err: e.1,
        }
    }
}

/// The error type
#[derive(Debug, Fail)]
pub enum Error {
    /// The request resulted in an error.
    #[fail(display = "JsonRpc {} failed", rpc_name)]
    JsonRpc {
        rpc_name: &'static str,
        #[cause]
        err: jsonrpc::Error,
    },
    /// The received response format is malformed.
    #[fail(display = "JsonRpc {} response format is invalid", rpc_name)]
    MalformedResponse { rpc_name: &'static str },
}
