//! Blockchain related RPC result types.

/// Models the result of "waitfornewblock", and "waitforblock"
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockRef {
    // TODO: Use Sha256dHash
    pub hash: String,
    pub height: u64,
}
