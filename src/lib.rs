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

use std::fmt::{self, Display, Formatter};

use jsonrpc::client::Client;
use strason::Json;

use bitcoin::blockdata::transaction::Transaction;
use bitcoin::network::serialize as bitcoin_ser;
use bitcoin::util::hash::Sha256dHash;

use bitcoin_rpc_json::*;

use failure::ResultExt;

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
        pub fn $rpc_method:ident(&self) -> RpcResult<$ty:ty>
    ) => {
        $(#[$outer:meta])*
        pub fn $rpc_method(&self) -> $crate::RpcResult<$ty> {
            let v: $ty = rpc_request!(&self.client,
                                        stringify!($rpc_method).to_string(),
                                        vec![]);
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
    
    rpc_method!(pub fn getconnectioncount(&self) -> RpcResult<u64>);
    rpc_method!(pub fn ping(&self) -> RpcResult<()>);
    rpc_method!(pub fn getnetworkinfo(&self) -> RpcResult<net::NetworkInfo>);
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
