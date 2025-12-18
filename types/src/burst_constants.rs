// Burst-specific constants that are easily configurable
// These can be changed later without modifying core logic

/// BRN increment rate per minute
/// Formula: BRN = (time_since_verification_in_minutes * BRN_INCREMENT_RATE_PER_MINUTE) - total_TRST_ever_created
pub const BRN_INCREMENT_RATE_PER_MINUTE: u64 = 1;

/// TRST expiry period in years (globally the same for all TRST tokens)
pub const TRST_EXPIRY_YEARS: u64 = 70;

/// Number of seconds in a year (approximate)
const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;

/// TRST expiry period in seconds
pub fn trst_expiry_seconds() -> u64 {
    TRST_EXPIRY_YEARS * SECONDS_PER_YEAR
}

/// Threshold for circle validators (number of circle validators needed)
pub const CIRCLE_VALIDATOR_THRESHOLD: u32 = 5;

/// Threshold for random validators (number of random validators needed)
pub const RANDOM_VALIDATOR_THRESHOLD: u32 = 10;

/// Minimum stake amount for circle validators (in BRN points to burn)
/// This should be a substantial amount to prove confidence
pub const MIN_CIRCLE_VALIDATOR_STAKE_BRN: u64 = 1000;

/// Minimum stake amount for random validators (in TRST tokens to stake)
/// This should be a substantial amount to prove confidence
pub const MIN_RANDOM_VALIDATOR_STAKE_TRST: u64 = 1000;

/// Account prefix for Burst (changed from "nano" to "brst")
pub const ACCOUNT_PREFIX: &str = "brst";

// Network Discovery Constants
// IMPORTANT: These prevent BURST nodes from connecting to Nano nodes

/// Network identifier for BURST Live Network
/// Changed from Nano's 0x5243 ('R','C') to 0x4252 ('B','R' for BURST)
/// This ensures BURST nodes only connect to other BURST nodes
pub const NETWORK_IDENTIFIER_BURST_LIVE: u16 = 0x4252; // 'B', 'R'

/// Network identifier for BURST Test Network
/// Changed from Nano's 0x5258 ('R','X') to 0x4254 ('B','T' for BURST Test)
pub const NETWORK_IDENTIFIER_BURST_TEST: u16 = 0x4254; // 'B', 'T'

/// Network identifier for BURST Dev Network
/// Changed from Nano's 0x5241 ('R','A') to 0x4244 ('B','D' for BURST Dev)
pub const NETWORK_IDENTIFIER_BURST_DEV: u16 = 0x4244; // 'B', 'D'

/// Preconfigured peer addresses for BURST Live Network
/// IMPORTANT: Set to empty array initially - users must configure peers manually
/// or set up their own peering infrastructure. This prevents accidental connections
/// to Nano nodes.
pub const PRECONFIGURED_PEERS_BURST_LIVE: [&'static str; 0] = [];

/// Preconfigured peer addresses for BURST Test Network
pub const PRECONFIGURED_PEERS_BURST_TEST: [&'static str; 0] = [];

/// Preconfigured peer addresses for BURST Dev Network
pub const PRECONFIGURED_PEERS_BURST_DEV: [&'static str; 0] = [];

/// Default node port for BURST (different from Nano's 7075)
pub const DEFAULT_PORT_NODE_BURST: u16 = 7077;

/// Default RPC port for BURST (different from Nano's 7076)
pub const DEFAULT_PORT_RPC_BURST: u16 = 7077;

