# akropolis-node

## Example

First node:

```bash
./target/release/akropolis-node --rpc-external --ws-external --name Foo --chain akropolis --base-path ./blocks --telemetry-url ws://localhost:1024 --node-key 000000000000000000000000000000000000000000000000000000000000F001 --key akropolis --validator --dev
```

Second node:

```bash
./target/release/akropolis-node --port 30339 --name Bar --chain akropolis --base-path ./blocks2 --key Bar --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmVCF4PSBgj2ya6n6TvzoTiV6Gbzf6eXVQBQ2Yi6GpVkBF --telemetry-url ws://localhost:1024 --dev
```

Third node:

```bash
./target/release/akropolis-node --port 30999 --name Faz --chain akropolis --base-path ./blocks3 --key Faz --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmVCF4PSBgj2ya6n6TvzoTiV6Gbzf6eXVQBQ2Yi6GpVkBF --telemetry-url ws://localhost:1024 --dev
```
