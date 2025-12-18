<p style="text-align:center;"><img src="/doc/images/logo.svg" width"300px" height="auto" alt="Logo"></p>


[![Unit Tests](https://github.com/simpago/rsnano-node/actions/workflows/unit_tests.yml/badge.svg)](https://github.com/simpago/rsnano-node/actions/workflows/unit_tests.yml)
[![codecov](https://codecov.io/gh/rsnano-node/rsnano-node/graph/badge.svg?token=LIATNV5NBP)](https://codecov.io/gh/rsnano-node/rsnano-node)
[![Discord](https://img.shields.io/badge/discord-join%20chat-orange.svg)](https://discord.gg/kBwvAyxEWE)


### What is BURST?

BURST is a cryptocurrency node implementation written in Rust, based on the Nano codebase but modified for the BURST ecosystem.

### What is BURST?

BURST is a digital payment protocol designed to be accessible and lightweight, 
with a focus on removing inefficiencies present in other cryptocurrencies. 
With ultrafast transactions and zero fees on a secure, green and decentralized 
network, BURST features BRN (Burn Points) and TRST (Trust Tokens) with humanity verification.

### Links & Resources

* [RsNano Website](https://rsnano.com)
* [Discord Chat](https://discord.gg/kBwvAyxEWE)
* [Twitter](https://twitter.com/gschauwecker)

### Installation

## Option 1: Build from source

Currently you can build BURST on Linux and on Mac.

To just build and run the burst_node:

    cd burst_node/main
    cargo build --release
    cargo run --release -- --network=dev --data-path=../burst_data node run

To install and run the burst_node executable:

    cd burst_node
    cargo install --path main
    burst_node --network=dev --data-path=./burst_data node run

## Running it with a GUI

You can run a BURST node with a GUI:

    cd burst_node/tools/insight
    cargo run --release

### Contact us

We want to hear about any trouble, success, delight, or pain you experience when
using RsNano. Let us know by [filing an issue](https://github.com/simpago/rsnano-node/issues), or joining us on [Discord](https://discord.gg/kBwvAyxEWE).

# The codebase

Have a look at the [AI generated documentation of the codebase](https://deepwiki.com/rsnano-node/rsnano-node).

The Rust code is structured according to A-frame architecture and is built with nullable infrastructure. 
This design and testing approach is [extensively documented on James Shore's website](http://www.jamesshore.com/v2/projects/nullables/testing-without-mocks)

Watch James Shore's presentation of nullables on YouTube: [Testing Without Mocks - James Shore | Craft Conference 2024](https://www.youtube.com/watch?v=GjZg6lDBKkk)

The following diagram shows how the crates are organized. The crates will be split up more when the codebase grows.

![crate diagram](http://www.plantuml.com/plantuml/proxy?cache=no&fmt=svg&src=https://raw.github.com/rsnano-node/rsnano-node/develop/doc/crates.puml)

* `main`: The node executable.
* `daemon`: Starts the node and optionally the RPC server.
* `node`:The node implementation.
* `rpc_server`: Implemenation of the RPC server.
* `websocket_server`: Implemenation of the websocket server.
* `wallet`: Wallet implementation. It manages multiple wallets which can each have multiple accounts.
* `ledger`: Ledger implementation. It is responsible for the consinstency of the data stores.
* `store_lmdb`: LMDB implementation of the data stores.
* `messages`: Message types that nodes use for communication.
* `network`: Manage outbound/inbound TCP channels to/from other nodes.
* `work`: Proof of work generation via CPU or GPU
* `types`: Contains the basic types like `BlockHash`, `Account`, `KeyPair`,...
* `utils`: Contains utilities like stats
* `nullables`: Nullable wrappers for infrastructure libraries.

