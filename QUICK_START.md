# BURST Node Quick Start

Get your BURST node running in 5 minutes!

## 1. Build the Node

```bash
cd rsnano-node
cargo build --release
```

⏱️ First build: ~15-30 minutes  
⏱️ Subsequent builds: ~2-5 minutes

## 2. Initialize and Start

```bash
# Initialize (creates genesis block)
cd main
cargo run --release -- --network=dev --data-path=../burst_data node initialize

# Start the node
cargo run --release -- --network=dev --data-path=../burst_data node run
```

## 3. Test It Works

In another terminal:

```bash
# Quick test
./test_burst.sh

# Or manually test
curl -X POST http://localhost:7077 \
  -H "Content-Type: application/json" \
  -d '{"action": "version"}'
```

## 4. Connect Wallet

1. Start Flutter wallet: `cd burst_wallet && flutter run`
2. Go to Node Settings
3. Connect to: `http://localhost:7077`
4. Create an account and test!

## Common Issues

**Port already in use?**
```bash
# Find what's using port 7077
lsof -i :7077
# Kill it or change port in config
```

**Build errors?**
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

**RPC not responding?**
- Check `config.toml` has `[rpc] enable = true`
- Check firewall settings
- Verify node is running (check logs)

## Next Steps

- Read `TESTING_GUIDE.md` for detailed testing
- Read `NODE_DISCOVERY.md` for network setup
- Check `burst_wallet/README.md` for wallet usage

