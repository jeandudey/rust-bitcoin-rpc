<p align="center">
  <a href="https://travis-ci.org/jeandudey/bitcoin-rpc">
    <img src="https://travis-ci.org/jeandudey/bitcoin-rpc.svg?branch=master" alt="Build Status">
    </img>
  </a>

  <a href="https://crates.io/crates/bitcoin-rpc">
    <img src="https://img.shields.io/crates/v/bitcoin-rpc.svg?maxAge=2592000" alt="Crates.io Version">
    </img>
  </a>

  <br/>

   <strong>
     <a href="https://docs.rs/bitcoin-rpc">
       Documentation
     </a>
   </strong>
</p>

# Bitcoin RPC
This crate implements an Bitcoin JSON-RPC client in rust, this cate doesn't
intends to be a complete implementation of all the bitcoin rpc methods so if you
need some method you can create a pull request for it.

## Usage
Add the dependency to your `Cargo.toml`:
```toml
[dependencies]
bitcoin-rpc = "0.2"
```

And this to your crate root:
```rust
extern crate bitcoin_rpc;
```
