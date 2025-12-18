# BURST Node Discovery and Peer Connection

## How Nodes Find Each Other

BURST nodes discover and connect to each other through several mechanisms:

### 1. **Network Identifier Protection**
- Each network has a unique **network identifier** (2 bytes)
- BURST uses different identifiers than Nano:
  - BURST Live: `0x4252` ('B','R')
  - BURST Test: `0x4254` ('B','T')
  - BURST Dev: `0x4244` ('B','D')
- Nodes **only connect to peers with matching network identifiers**
- This prevents BURST nodes from accidentally connecting to Nano nodes

### 2. **Preconfigured Peers (Initial Bootstrap)**
- Nodes start with a list of **preconfigured peer addresses**
- For BURST, this is currently **empty by default** (to prevent Nano connections)
- Users must manually configure initial peers in the node config file
- Example config:
  ```toml
  [node]
  preconfigured_peers = ["192.168.1.100:7077", "example.com:7077"]
  ```

### 3. **Peer Cache (Persistent Storage)**
- Once connected, nodes **store discovered peers in a database**
- On restart, nodes load peers from cache and try to reconnect
- Location: `{data_dir}/peers.ldb`
- This allows nodes to "remember" peers across restarts

### 4. **Keepalive Messages (Peer Exchange)**
- Nodes periodically send **keepalive messages** to connected peers
- Keepalive messages contain:
  - List of known peer addresses
  - Node's own peering address
- When a node receives a keepalive, it:
  - Learns about new peers from the list
  - Stores them in the peer cache
  - May attempt to connect to new peers

### 5. **Bootstrap Process**
- When a node needs to sync the ledger, it uses **bootstrap**
- Bootstrap requests blocks from peers
- During bootstrap, nodes discover additional peers
- Peers discovered during bootstrap are added to the cache

## Configuration

### Setting Initial Peers

Edit your node configuration file (`config.toml`):

```toml
[node]
# List of initial peer addresses to connect to
# Format: "hostname:port" or "ip:port"
preconfigured_peers = [
    "192.168.1.100:7077",
    "burst-node.example.com:7077",
    "[2001:db8::1]:7077"  # IPv6 example
]
```

### Manual Peer Management

You can also manage peers via RPC:

```bash
# Add a peer
curl -X POST http://localhost:7077 \
  -H "Content-Type: application/json" \
  -d '{"action": "work_peer_add", "address": "192.168.1.100", "port": 7077}'

# List peers
curl -X POST http://localhost:7077 \
  -H "Content-Type: application/json" \
  -d '{"action": "peers"}'
```

## Network Isolation

### Why BURST Won't Connect to Nano

1. **Different Network Identifiers**: BURST uses `0x4252` vs Nano's `0x5243`
2. **Different Default Ports**: BURST uses port `7077` vs Nano's `7075`
3. **No Preconfigured Nano Peers**: BURST doesn't have `peering.nano.org` in its list
4. **Protocol Validation**: Nodes validate network identifiers during handshake

### Verification

To verify your node is on the BURST network:

```bash
# Check node ID (should show BURST network identifier)
curl -X POST http://localhost:7077 \
  -H "Content-Type: application/json" \
  -d '{"action": "node_id"}'
```

## Starting a New Network

When starting a brand new BURST network:

1. **First Node**: 
   - Start with empty `preconfigured_peers`
   - This node will be isolated until others connect to it

2. **Additional Nodes**:
   - Configure `preconfigured_peers` with the first node's address
   - They will connect and discover each other via keepalive

3. **Network Growth**:
   - As more nodes join, they share peer lists via keepalive
   - The network becomes more resilient and decentralized

## Troubleshooting

### Node Can't Find Peers

1. **Check Network Identifier**: Ensure all nodes use the same network identifier
2. **Check Firewall**: Ensure port 7077 (or your configured port) is open
3. **Check Preconfigured Peers**: Verify peer addresses are correct and reachable
4. **Check Logs**: Look for connection errors in node logs

### Node Connects to Wrong Network

1. **Verify Network Identifier**: Check `config.toml` matches BURST constants
2. **Check Preconfigured Peers**: Ensure no Nano peer addresses are listed
3. **Clear Peer Cache**: Delete `peers.ldb` and restart with correct config

## Security Considerations

- **Only connect to trusted peers** initially
- **Verify peer addresses** before adding them
- **Monitor connections** to detect suspicious activity
- **Use firewall rules** to restrict incoming connections if needed

