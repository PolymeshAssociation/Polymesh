/// Money matters.
pub mod currency {
    use primitives::Balance;
    // TODO: Define proper units. These are placeholders.
    pub const POLY: Balance = 1_000_000_000_000;
    pub const DOLLARS: Balance = POLY / 100;
    pub const CENTS: Balance = DOLLARS / 100;
    pub const MILLICENTS: Balance = CENTS / 1_000;
    pub const ONE_UNIT: Balance = 1_000_000;
    pub const MAX_SUPPLY: Balance = ONE_UNIT * 1_000_000_000_000;
}

/// Time and blocks.
pub mod time {
    use primitives::{BlockNumber, Moment};
    // Kusama & mainnet
    pub const MILLISECS_PER_BLOCK: Moment = 6000;
    // Testnet
    //	pub const MILLISECS_PER_BLOCK: Moment = 1000;
    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;
    // Kusama & mainnet
    pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 4 * HOURS;
    // Testnet
    //	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;

    // These time units are defined in number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;

    // 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
    pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
}

/// Fee-related.
pub mod fee {
    pub use sr_primitives::Perbill;

    /// The block saturation level. Fees will be updates based on this value.
    pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);
}

/// DID-related.
pub mod did {
    /// prefix for user dids
    pub const USER: [u8; 5] = *b"USER:";
}

// ERC1400 transfer status codes
pub const ERC1400_TRANSFER_FAILURE: u8 = 0x50;
pub const ERC1400_TRANSFER_SUCCESS: u8 = 0x51;
pub const ERC1400_INSUFFICIENT_BALANCE: u8 = 0x52;
pub const ERC1400_INSUFFICIENT_ALLOWANCE: u8 = 0x53;
pub const ERC1400_TRANSFERS_HALTED: u8 = 0x54;
pub const ERC1400_FUNDS_LOCKED: u8 = 0x55;
pub const ERC1400_INVALID_SENDER: u8 = 0x56;
pub const ERC1400_INVALID_RECEIVER: u8 = 0x57;
pub const ERC1400_INVALID_OPERATOR: u8 = 0x58;

// Application-specific status codes
pub const APP_NOT_AFFECTED: u8 = 0xa0;
pub const APP_SUCCESS: u8 = 0xa1;
pub const APP_MAX_HOLDERS_REACHED: u8 = 0xa2;
pub const APP_MANUAL_APPROVAL_EXPIRED: u8 = 0xa3;
pub const APP_FUNDS_LIMIT_REACHED: u8 = 0xa4;
pub const APP_TX_VOLUME_LIMIT_REACHED: u8 = 0xa5;
pub const APP_BLACKLISTED_TX: u8 = 0xa6;
pub const APP_FUNDS_LOCKED: u8 = 0xa7;
pub const APP_INVALID_GRANULARITY: u8 = 0xa8;
