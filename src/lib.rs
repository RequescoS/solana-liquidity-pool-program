pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

pub const POOL_SEED: &str = "liquidity pool";
solana_program::declare_id!("78yZvMzqAFzSHJrLNVWfqLRFFQ5ZCGzNXB4PBxmp6z5Y");
