# BURST Node Testing Guide

This guide will help you build, configure, and test your BURST node.

## Prerequisites

- Rust toolchain (install from https://rustup.rs/)
- Linux or macOS (Windows support may be limited)
- At least 4GB RAM recommended
- Disk space for blockchain data

## Step 1: Build the Node

```bash
cd rsnano-node
cargo build --release
```

This will compile the entire node. The first build may take 10-30 minutes.

## Step 2: Generate Configuration

Generate a default configuration file:

```bash
cd main
cargo run --release -- node generate-config node > ../config.toml
```

Or generate RPC config:

```bash
cargo run --release -- rpc generate-config > ../rpc_config.toml
```

## Step 3: Configure for BURST

Edit `config.toml` and make these changes:

```toml
[node]
# Use dev network for testing (isolated from live/test)
# Note: You may need to modify the code to add a BURST network type
peering_port = 7077  # Different from Nano's 7075
allow_local_peers = true  # Allow connections on localhost

# IMPORTANT: Set preconfigured peers to empty or your test nodes
preconfigured_peers = []

[rpc]
enable = true
port = 7077  # Different from Nano's 7076
address = "::"  # Listen on all interfaces
enable_control = true  # Enable RPC commands
```

## Step 4: Initialize the Node

Initialize the data directory with the genesis block:

```bash
cd main
cargo run --release -- --network=dev --data-path=../burst_data node initialize
```

This creates the genesis block and initializes the database.

## Step 5: Start the Node

Start the node:

```bash
cd main
cargo run --release -- --network=dev --data-path=../burst_data node run
```

You should see output like:
```
Starting BURST node...
Listening on port 7077
RPC server listening on port 7077
```

## Step 6: Test Basic Functionality

### Test 1: Check Node Status

```bash
curl -X POST http://localhost:7077 \
  -H "Content-Type: application/json" \
  -d '{"action": "version"}'
```

Expected response:
```json
{
  "node_vendor": "rsnano",
  "version": "...",
  "rpc_version": "..."
}
```

### Test 2: Create an Account

```bash
curl -X POST http://localhost:7077 \
  -H "Content-Type: application/json" \
  -d '{"action": "account_create"}'
```

This returns a new account address (should start with `brst_`).

### Test 3: Get BRN Balance (BURST-specific)

```bash
curl -X POST http://localhost:7077 \
  -H "Content-Type: application/json" \
  -d '{
    "action": "brn_balance",
    "account": "brst_1111111111111111111111111111111111111111111111111111111111111111111111"
  }'
```

Expected response:
```json
{
  "brn": "0"
}
```

### Test 4: Request Verification (BURST-specific)

```bash
curl -X POST http://localhost:7077 \
  -H "Content-Type: application/json" \
  -d '{
    "action": "request_verification",
    "account": "brst_YOUR_ACCOUNT_HERE"
  }'
```

### Test 5: Get TRST History (BURST-specific)

```bash
curl -X POST http://localhost:7077 \
  -H "Content-Type: application/json" \
  -d '{
    "action": "trst_history",
    "account": "brst_YOUR_ACCOUNT_HERE",
    "count": 10
  }'
```

## Step 7: Run Unit Tests

Run all tests:

```bash
cd rsnano-node
cargo test --release
```

Run BURST-specific tests:

```bash
cargo test --package rsnano_ledger --lib burst_tests
```

## Step 8: Test with Multiple Nodes

### Node 1 (Terminal 1):

```bash
cd rsnano-node/main
cargo run --release -- --network=dev --data-path=../burst_data1 node run
```

### Node 2 (Terminal 2):

First, get Node 1's IP and port, then:

```bash
cd rsnano-node/main
# Edit config.toml to add Node 1 as a peer:
# preconfigured_peers = ["127.0.0.1:7077"]

cargo run --release -- --network=dev --data-path=../burst_data2 node run
```

## Step 9: Test BURST Wallet Connection

1. Start the Flutter wallet:
   ```bash
   cd burst_wallet
   flutter run
   ```

2. In the wallet, go to Node Settings
3. Connect to: `http://localhost:7077`
4. Create an account
5. Test BRN balance display
6. Test verification request

## Troubleshooting

### Node Won't Start

1. **Check port availability**:
   ```bash
   lsof -i :7077
   ```

2. **Check data directory permissions**:
   ```bash
   ls -la burst_data/
   ```

3. **Check logs** for error messages

### RPC Not Responding

1. **Verify RPC is enabled** in config.toml
2. **Check firewall** settings
3. **Test with curl** from command line first

### Can't Connect Nodes

1. **Verify network identifier** matches (both nodes must use same network)
2. **Check preconfigured_peers** configuration
3. **Check firewall** allows port 7077
4. **Check logs** for connection errors

### BURST Endpoints Not Found

1. **Verify you built the latest code** with BURST changes
2. **Check RPC handler** includes BURST commands
3. **Check action name** matches exactly (case-sensitive)

## Quick Test Script

Save this as `test_burst.sh`:

```bash
#!/bin/bash

NODE_URL="http://localhost:7077"

echo "Testing BURST Node at $NODE_URL"
echo ""

echo "1. Testing version..."
curl -s -X POST $NODE_URL \
  -H "Content-Type: application/json" \
  -d '{"action": "version"}' | jq '.'

echo ""
echo "2. Creating account..."
ACCOUNT=$(curl -s -X POST $NODE_URL \
  -H "Content-Type: application/json" \
  -d '{"action": "account_create"}' | jq -r '.account')

echo "Account: $ACCOUNT"
echo ""

echo "3. Getting BRN balance..."
curl -s -X POST $NODE_URL \
  -H "Content-Type: application/json" \
  -d "{\"action\": \"brn_balance\", \"account\": \"$ACCOUNT\"}" | jq '.'

echo ""
echo "4. Requesting verification..."
curl -s -X POST $NODE_URL \
  -H "Content-Type: application/json" \
  -d "{\"action\": \"request_verification\", \"account\": \"$ACCOUNT\"}" | jq '.'

echo ""
echo "5. Getting TRST history..."
curl -s -X POST $NODE_URL \
  -H "Content-Type: application/json" \
  -d "{\"action\": \"trst_history\", \"account\": \"$ACCOUNT\"}" | jq '.'
```

Make it executable and run:
```bash
chmod +x test_burst.sh
./test_burst.sh
```

## Next Steps

1. **Test verification flow** with multiple validators
2. **Test burn transactions** (BRN → TRST)
3. **Test send transactions** (TRST transfers)
4. **Test merge/split** transactions
5. **Test expiry** logic (70-year expiry)
6. **Test orphaning** logic (illegitimate tokens)

## Useful Commands

```bash
# Check node status
curl -X POST http://localhost:7077 -H "Content-Type: application/json" -d '{"action": "version"}'

# List peers
curl -X POST http://localhost:7077 -H "Content-Type: application/json" -d '{"action": "peers"}'

# Get account info
curl -X POST http://localhost:7077 -H "Content-Type: application/json" -d '{"action": "account_info", "account": "brst_..."}'

# Stop the node
# Press Ctrl+C or send SIGTERM
```

## Development Tips

1. **Use `--release` flag** for production-like performance
2. **Use separate data directories** for different test scenarios
3. **Check logs** regularly for errors
4. **Test incrementally** - verify each feature before moving on
5. **Use `cargo test`** frequently during development

