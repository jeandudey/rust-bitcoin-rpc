//    Rust Bitcoin RPC API client.
//    Copyright (C) 2016  Jean Pierre De Jesus Dudey Diaz
//
//    This program is free software: you can redistribute it and/or modify
//    it under the terms of the GNU General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    This program is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//    GNU General Public License for more details.
//
//    You should have received a copy of the GNU General Public License
//    along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

#[macro_use]
extern crate jsonrpc_v1;
extern crate strason;
extern crate serde;

use jsonrpc_v1::client::Client as RpcClient;
use jsonrpc_v1::Error as RpcError;
use strason::Json;

macro_rules! rpc_method {
    ($method_name:ident<$return_type:ty>) => {
        pub fn $method_name(&self) -> Result<$return_type, RpcError> {
            let request = self.client.build_request(String::from(stringify!($method_name)), vec![]);

            match self.client.send_request(&request).and_then(|res| res.into_result::<$return_type>()) {
                Ok(res) => return Ok(res),
                Err(e) => return Err(e),
            }
        }
    };
    ($method_name:ident<$return_type:ty>, { $($param:ident : $param_ty:ty),* }) => {
        pub fn $method_name(&self, $($param : $param_ty),*) -> Result<$return_type, RpcError> {
            let mut params: Vec<Json> = Vec::new();

            $(
                params.push(Json::from($param));
            )*

            let request = self.client.build_request(String::from(stringify!($method_name)), params);

            match self.client.send_request(&request).and_then(|res| res.into_result::<$return_type>()) {
                Ok(res) => return Ok(res),
                Err(e) => return Err(e),
            }
        }
    }
}

/// A Handle to a Bitcoin Rpc connection
pub struct BitcoinRpc {
    client: RpcClient,
}

pub struct SerializedBlock {
    pub result: String,
}

pub struct Block {
    pub hash: String,
    pub confirmations: i64,
    pub size: i64,
    pub height: i64,
    pub version: i64,
    pub merkleroot: String,
    pub tx: Vec<Json>,
    pub txid: String,
    pub time: i64,
    pub nonce: i64,
    pub bits: String,
    pub chainwork: String,
    pub previousblockhash: Option<String>,
    pub nextblockhash: Option<String>,
}

pub enum GetBlockReply {
    True(Block),
    False(SerializedBlock),
}

serde_struct_enum_impl!(GetBlockReply,
                        True, Block, hash, confirmations, size, height, version, merkleroot, tx, txid <- "TXID", time, nonce, bits,  chainwork, previousblockhash, nextblockhash;
                        False, SerializedBlock, result
);

pub struct Enforce {
    pub status: bool,
    pub found: i64,
    pub required: i64,
    pub window: i64,
}

serde_struct_impl!(Enforce, status, found, required, window);


pub struct Reject {
    pub status: bool,
    pub found: i64,
    pub required: i64,
    pub window: i64,
}

serde_struct_impl!(Reject, status, found, required, window);

pub struct Softfork {
    pub id: String,
    pub version: i64,
    pub enforce: Enforce,
    pub reject: Reject,
}

serde_struct_impl!(Softfork, id, version, enforce, reject);

pub struct BlockChainInfo {
    pub chain: String,
    pub blocks: i64,
    pub headers: i64,
    pub bestblockhash: String,
    pub difficulty: f64,
    pub mediantime: i64,
    pub verificationprogress: f64,
    pub chainwork: String,
    pub pruned: bool,
    pub softforks: Vec<Softfork>,
}

serde_struct_impl!(BlockChainInfo,
                   chain,
                   blocks,
                   headers,
                   bestblockhash,
                   difficulty,
                   mediantime,
                   verificationprogress,
                   chainwork,
                   pruned,
                   softforks);


pub struct Tip {
    pub height: u64,
    pub hash: String,
    pub branchlen: u64,
    pub status: String,
}

serde_struct_impl!(Tip, height, hash, branchlen, status);

pub struct MemPoolInfo {
    pub size: i64,
    pub bytes: i64,
    pub usage: i64,
    pub maxmempool: i64,
    pub mempoolminfee: f64,
}

serde_struct_impl!(MemPoolInfo, size, bytes, usage, maxmempool, mempoolminfee);

pub struct TxDescription {
    pub txid: String,
    pub size: i64,
    pub fee: f64,
    pub time: i64,
    pub height: i64,
    pub startingpriority: i64,
    pub currentpriority: i64,
    pub depends: Vec<String>,
}

pub struct TXIDS {
    pub result: Vec<String>,
}

pub enum RawMemPool {
    True(TxDescription),
    False(TXIDS),
}

serde_struct_enum_impl!(RawMemPool,
                        True, TxDescription, txid <- "TXID", size, fee, time, height, startingpriority, currentpriority, depends;
                        False, TXIDS, result
);

pub struct ScriptPubKey {
    pub asm: String,
    pub hex: String,
    pub reqsigs: i64,
    pub scripttype: String,
    pub addresses: Vec<String>,
}

serde_struct_impl!(ScriptPubKey,
                   asm,
                   hex,
                   reqsigs <- "regSigs",
                   scripttype <- "type",
                   addresses);

pub struct TxOut {
    pub bestblock: String,
    pub confirmations: i64,
    pub value: f64,
    pub scriptpubkey: ScriptPubKey,
    pub version: i64,
    pub coinbase: bool,
}

serde_struct_impl!(TxOut,
                   bestblock,
                   confirmations,
                   value,
                   scriptpubkey <- "scriptPubKey",
                   version,
                   coinbase);


pub struct TxOutSetInfo {
    pub height: i64,
    pub bestblock: String,
    pub transactions: i64,
    pub txouts: i64,
    pub bytes_serialized: i64,
    pub hash_serialized: String,
    pub total_amount: f64,
}

serde_struct_impl!(TxOutSetInfo,
                   height,
                   bestblock,
                   transactions,
                   txouts,
                   bytes_serialized,
                   hash_serialized,
                   total_amount);

impl BitcoinRpc {
    /// Creates a connection to a bitcoin rpc server
    pub fn new(url: &str, user: Option<String>, pass: Option<String>) -> Self {
        // Check that if we have a password, we have a username; other way around is ok
        debug_assert!(pass.is_none() || user.is_some());

        BitcoinRpc { client: RpcClient::new(String::from(url), user, pass) }
    }

    rpc_method!(getbestblockhash<String>);

    rpc_method!(getblock<GetBlockReply>, {
        header_hash: String,
        format: bool
    });

    rpc_method!(getblockchaininfo<BlockChainInfo>);
    rpc_method!(getblockcount<i64>);

    rpc_method!(getblockhash<String>, {
        block_height: i64
    });

    rpc_method!(getchaintips<Vec<Tip> >);
    rpc_method!(getdifficulty<f64>);
    rpc_method!(getmempoolinfo<MemPoolInfo>);

    rpc_method!(getrawmempool<RawMemPool>, {
        format: bool
    });

    rpc_method!(gettxout<TxOut>, {
        txid: String,
        vout: i64,
        unconfirmed: bool
    });

    rpc_method!(gettxoutsetinfo<TxOutSetInfo>);
}
