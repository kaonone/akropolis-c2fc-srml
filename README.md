# Akropolis C2FC

Implementation of financial primitive Commitments to Future Cashflows on Substrate. C2FC is a new financial primitive and a DeFi equivalent of cashflow financing. C2FC bridges traditional finance and Web3 by providing DeFi and Web3 entrepreneurs with capital to grow. For more details read about C2FC  [here](https://wiki.akropolis.io/c2fc/overview/).


## Try it out

- [telemetry](https://telemetry.polkadot.io/#/Akropolis)
    - [bootnode #1](https://telemetry.polkadot.io/network_state/Akropolis/565/)
    - [bootnode #2](https://telemetry.polkadot.io/network_state/Akropolis/569/)
- [polkadot.js.org](https://polkadot.js.org/apps/)

Connect to local node:
*  Go to [admin tools / settings](https://polkadot.js.org/apps/#/settings).
*  Select __remote node/endpoint to connect to__ => `Local Node`.
*  Press `Save & Reload`


## Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```


## Setup Node


```bash
#!/usr/bin/env bash
set -e
mkdir akropolis
cd akropolis
git clone https://github.com/akropolisio/akropolis-polkadot.git ./node
# or git clone git@github.com:akropolisio/akropolis-polkadot.git ./node
cd node
# Install required tools:
./scripts/init.sh
# Build the WebAssembly binary:
./scripts/build.sh
# Build all native code and install:
cargo install --force --path ./ akropolis
cd ..
# Build chain-spec and export:
akropolis build-spec --chain akropolis > spec.json
```


## Run Node


```bash
#!/usr/bin/env bash

akropolis --chain akropolis \
    --telemetry-url wss://telemetry.polkadot.io/submit/ \
    --rpc-cors all \
    --name "AkropolisC2FC-NodeName" \
    --base-path ./base \
    --keystore-path ./keystore \
    --listen-addr /ip4/0.0.0.0/tcp/30333 \
    --bootnodes \
          /ip4/178.128.225.241/tcp/30333/p2p/QmP71M5Xami5ud3ZL9SZe1Wic7xPBPxaAsV4W3wZe9KWCK \
          /ip4/157.230.35.215/tcp/30333/p2p/QmRe8UD9o8F8Jv55JQvrmNPt5SaWYU2fbey2HoiLzLYawo
```


## How it works

### Creation of C2FC

Prerequisites:
You should have accounts with positive balance to test creation / transfer / exchange of C2FC (let's create this account with seed `//Alice`, `//Bob`, `//Charlie`).


Core elements of C2FC on Substrate implementation:

- User accounts
- FreePromise
- Promise
- Bucket
- Resitrar (Storage-module C2FC)

Creation of C2FC consists of two steps: at first, you create *Free Promise* and then assign a payee to it (*Promise* stage). While C2FC is in the *Free Promise stage*, it hasn’t recipient of payments (issuer is a recipient).



- go to [Extrinsics](https://polkadot.js.org/apps/#/extrinsics)
- Alice creates Bucket:
    - select __using the selected account__ => Alice
    - select __submit the following extrinsic__ `C2FC` :: `createBucket()`
    - `Submit Transaction`

- Bob creates Promise:
    - select __using the selected account__ => Bob
    - select __submit the following extrinsic__ `C2FC` :: `createPromise(value, period)` where
        - `value`: amount of regular payment
        - `period`: periodicity of regular payment
    - `Submit Transaction`


- Bob creates temporary Promise:
    - select __using the selected account__ => Bob
    - select __submit the following extrinsic__ `C2FC` :: `createPromiseUntil(value, period, until)` where
        - `value`: amount of regular payment
        - `period`: periodicity of regular payment
        - `until`: date (block) of last payment
    - `Submit Transaction`

- Bob makes changes to Promise:
    - select __using the selected account__ => Bob
    - select __submit the following extrinsic__ `C2FC` :: `editPromise(promise_id, value, period)` where
        - `promise_id`: id (hash) of Bob's promise
        - `value`: amount of regular payment
        - `period`: periodicity of regular payment
    - `Submit Transaction`

- Bob stakes tokens in order to ___ the Promise:
    - select __using the selected account__ => Bob
    - select __submit the following extrinsic__ `C2FC` :: `stakeToPromise(promise_id, amount)` where
        - `promise_id`: id (hash) of Bob's promise
        - `amount`: size (amount) of stake
    - `Submit Transaction`
    - select __submit the following extrinsic__ `C2FC` :: `withdrawStaken(promise_id)` where
        - `promise_id`: id (hash) of Bob's promise
    - `Submit Transaction`

### Exchange of C2FC

- Alice sells his Bucket:
    - select __using the selected account__ => Alice
    - select __submit the following extrinsic__ `C2FC` :: `setPrice(bucket_id, new_price)` where
        - `bucket_id`: id (hash) of Bob's promise
        - `new_price`: price of the Bucket
    - `Submit Transaction`
    - select __using the selected account__ => Charlie
    - select __submit the following extrinsic__ `C2FC` :: `buyBucket(bucket_id, max_price)` where
        - `bucket_id`: id (hash) of Alice's promise
        - `max_price`: Charlie's maker price
    - `Submit Transaction`
- Alice transfers Bucket:
    - select __using the selected account__ => Alice
    - select __submit the following extrinsic__ `C2FC` :: `transfer(to, bucket_id)` where
        - `to`: receiver of Bucket (Charlie's account)
        - `bucket_id`: id (hash) of Alice's Bucket
    - `Submit Transaction`

- Alice is searching Bob's Promise:
    - go to [ChainState](https://polkadot.js.org/apps/#/chainstate)
    - select __selected state query__ => C2FC
    - use any method with name starts with `freePromise...`
        - `freePromisesCount(): u64`: number of open Promises
        - `freePromisesArray(u64): Hash`: get id (hash) of open Promise by increment
        - `freePromisesIndex(Hash): u64`: get increment number of open Promisy by it's id (hash)
        - `promises(Hash): FreePromise`: get Promise by id(hash)
    - click `+`

- Alica adds Bob's Promise to her Bucket:
    - select __using the selected account__ => Alice
    - select __submit the following extrinsic__ `C2FC` :: `acceptPromise(promise_id, bucket_id)` where:
        - `promise_id`: id (hash) of Bob's Promise
        - `bucket_id`: id (hash) of Alice's Bucket
    - `Submit Transaction`

### Pay for commitments

- Bob pays his commitments to Bucket Owner:
    - select __using the selected account__ => Bob
    - select __submit the following extrinsic__ `C2FC` :: `fillBucket(bucket_id, deposit)` where:
        - `bucket_id`: id (hash) of Alice's Bucket
        - `deposit`: funds, that trasferred from Bob's account to Bucket owner's account
    - `Submit Transaction`
