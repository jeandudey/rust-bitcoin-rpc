extern crate jsonrpc;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate strason;

extern crate bitcoin_amount;

use std::str::FromStr;

use jsonrpc::client::Client as RpcClient;

use bitcoin_amount::Amount;

/// A Handle to a Bitcoin JSON-RPC connection
pub struct BitcoinRpc {
    client: RpcClient,
}

impl BitcoinRpc {
    /// Creates a client to a bitcoind JSON-RPC server.
    pub fn new(url: &str, user: Option<String>, pass: Option<String>) -> Self {
        // Check that if we have a password, we have a username; other way
        // around is ok.
        debug_assert!(pass.is_none() || user.is_some());

        BitcoinRpc { client: RpcClient::new(String::from(url), user, pass) }
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
