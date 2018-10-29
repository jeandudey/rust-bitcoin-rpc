//! Blockchain related RPC result types.

use bitcoin::util::hash::Sha256dHash;
use strason::Json;

/// Models the result of "waitfornewblock", and "waitforblock"
#[derive(Debug, Clone)]
pub struct BlockRef {
    pub hash: Sha256dHash,
    pub height: u64,
}

impl From<SerdeBlockRef> for BlockRef {
    fn from(v: SerdeBlockRef) -> BlockRef {
        BlockRef {
            hash: Sha256dHash::from_hex(&v.hash).unwrap(),
            height: v.height,
        }
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SerdeBlockRef {
    pub hash: String,
    pub height: u64,
}

/// Models the result of "getblockchaininfo"
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockchainInfo {
    // TODO: Use Network from rust-bitcoin
    /// Current network name as defined in BIP70 (main, test, regtest)
    pub chain: String,
    /// The current number of blocks processed in the server
    pub blocks: u64,
    /// The current number of headers we have validated
    pub headers: u64,
    // TODO: Use Sha256dHash from rust-bitcoin
    /// The hash of the currently best block
    pub bestblockhash: String,
    /// The current difficulty
    pub difficulty: f64,
    /// Median time for the current best block
    pub mediantime: u64,
    /// Estimate of verification progress [0..1]
    pub verificationprogress: f64,
    /// Estimate of whether this node is in Initial Block Download mode
    pub initialblockdownload: bool,
    /// Total amount of work in active chain, in hexadecimal
    pub chainwork: String,
    /// The estimated size of the block and undo files on disk
    pub size_on_disk: u64,
    /// If the blocks are subject to pruning
    pub pruned: bool,
    /// Lowest-height complete block stored (only present if pruning is enabled)
    pub pruneheight: Option<u64>,
    /// Whether automatic pruning is enabled (only present if pruning is enabled)
    pub automatic_pruning: Option<bool>,
    /// The target size used by pruning (only present if automatic pruning is enabled)
    pub prune_target_size: Option<u64>,
    /// Status of softforks in progress
    pub softforks: Vec<Softfork>,
    // TODO: add a type?
    /// Status of BIP9 softforks in progress
    pub bip9_softforks: Json,
    /// Any network and blockchain warnings.
    pub warnings: String,
}

/// Status of a softfork
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Softfork {
    /// Name of softfork
    pub id: String,
    /// Block version
    pub version: u64,
    /// Progress toward rejecting pre-softfork blocks
    pub reject: RejectStatus,
}

/// Progress toward rejecting pre-softfork blocks
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RejectStatus {
    /// `true` if threshold reached
    pub status: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TxInInfoSignTx {
    #[serde(rename = "txid")]
    pub tx_id: String,
    pub vout: u32,
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key_hex: String,
    #[serde(rename = "redeemScript")]
    pub redeem_script_hex: String,
    #[serde(rename = "amount")]
    pub amount: f64
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TxInInfoCreateTx {
    #[serde(rename = "txid")]
    pub tx_id: String,
    pub vout: u32,
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key_hex: String,
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SignedRawTransaction {
    pub hex: String,
    pub complete: bool,
}

