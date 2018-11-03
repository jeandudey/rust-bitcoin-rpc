//! A very simple example used as a self-test of this library against a Bitcoin
//! Core node.
extern crate bitcoin_rpc;
#[macro_use]
extern crate failure;

use bitcoin_rpc::BitcoinRpc;

type Result<T> = std::result::Result<T, failure::Error>;

fn main() -> Result<()> {
    let mut args = std::env::args();

    let _exe_name = args.next().unwrap();

    let url = args
        .next()
        .ok_or_else(|| format_err!("Usage: <rpc_url> [username] [password]"))?;
    let user = args.next();
    let pass = args.next();

    let rpc = BitcoinRpc::new(url, user, pass);

    let best_block_hash = rpc.getbestblockhash()?;
    println!("best block hash: {}", best_block_hash);
    let bestblockcount = rpc.getblockcount()?;
    println!("best block height: {}", bestblockcount);
    let best_block_hash_by_height = rpc.get_blockhash(bestblockcount)?;
    println!("best block hash by height: {}", best_block_hash_by_height);
    assert_eq!(best_block_hash_by_height, best_block_hash);

    Ok(())
}
