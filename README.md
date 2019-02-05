# akropolis-node

## Start main node

### 1. Setup env on server

- install [rust](https://rustup.rs)
- install git, gcc, libssl-dev, clang

### 2. Clone repository

```bash
git clone git@github.com:akropolisio/akropolis-polkadot.git akropolis-c2fc
# or
git clone https://github.com/akropolisio/akropolis-polkadot.git akropolis-c2fc
```

Then build all

```bash
cd akropolis-c2fc
./build.all.sh
```

### 3. Run node

```bash
./akropolis-c2fc/target/release/akropolis-c2fc --rpc-external --ws-external --chain akropolis --validator --dev --listen-addr /ip4/1.2.3.4/tcp/30333 --name AkroMain
```

Specify `--base-path ./some-path` if you want to change directory of chain db.

Change `--telemetry-url ws://url` if you want to send telemetry to another service.

In `--listen-addr` replace `1.2.3.4` and `30333` to your public ip and port.

## How to run new node

### 1. Export chain spec

Login to server with already runned node.

```bash
./akropolis-c2fc/target/release/akropolis-c2fc build-spec --chain akropolis > chainspec.json
```

Check field "name" in `chainspec.json` it should be equal `"Akropolis"`.

### 2. Setup env on server

- install rust
- install git

### 3. Clone repository

```bash
git clone git@github.com:akropolisio/akropolis-polkadot.git akropolis-c2fc
# or
git clone https://github.com/akropolisio/akropolis-polkadot.git akropolis-c2fc
```

Then build all

```bash
cd akropolis-c2fc
./build.all.sh
```

### 4. Start with chain spec

Copy `chainspec.json` to that server.

And run node:

```bash
cd ~
./akropolis-c2fc/target/release/akropolis-c2fc --chain ./chainspec.json --dev --name AkropolisNodeName --telemetry-url wss://telemetry.polkadot.io/submit/
```

Specify `--base-path ./some-path` if you want to change directory of chain db.

Change `--telemetry-url ws://url` if you want to send telemetry to another service.

Specify `--listen-addr /ip4/0.1.2.3/tcp/30333` to setup custom listen ip address.
Replace `0.1.2.3` and `30333` to your public ip and port.
