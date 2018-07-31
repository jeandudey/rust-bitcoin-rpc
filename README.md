<p align="center">
  <a href="https://travis-ci.org/jeandudey/bitcoin-rpc">
    <img src="https://travis-ci.org/jeandudey/bitcoin-rpc.svg?branch=master" alt="Build Status">
    </img>
  </a>

  <a href="https://crates.io/crates/bitcoin-rpc">
    <img src="https://img.shields.io/crates/l/bitcoin-rpc.svg" alt="LicenseLicense">
    </img>
  </a>

  <a href="https://crates.io/crates/bitcoin-rpc">
    <img src="https://img.shields.io/crates/v/bitcoin-rpc.svg" alt="Crates.io Version">
    </img>
  </a>

  <a href="https://docs.rs/bitcoin-rpc">
    <img src="https://docs.rs/bitcoin-rpc/badge.svg" alt="Docs.rs">
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

Bitcoin JSON-RPC client implementation in Rust.

*This crate is not yet very stable, be careful.*

## Overview

*bitcoin-rpc* is a fast, safe JSON-RPC 2.0 client written in Rust.

*bitcoin-rpc* offers secure bitcoin money handling, using `strason`.

Be aware that this crate is not a complete implementation of all bitcoin
JSON-RPC methods available. This is due to the large effort needed to implement
all methods. If you need a method implemented feel free to open an issue or send
a Pull Request.

## Usage
Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
bitcoin-rpc = { git = "https://github.com/jeandudey/rust-bitcoin-rpc" }
```

And this to your crate root:
```rust
extern crate bitcoin_rpc;
```
