use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::{id, POOL_SEED};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct WithdrawedFee {
    pub user_x_withdraw: u64,
    pub user_y_withdraw: u64,
}

impl WithdrawedFee {
    pub fn get_withdraw_pubkey_with_bump(user: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[&user.to_bytes(), POOL_SEED.as_bytes()], &id())
    }

    pub fn get_withdraw_pubkey(user: &Pubkey) -> Pubkey {
        let (pubkey, _) = Self::get_withdraw_pubkey_with_bump(user);
        pubkey
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct TotalCommision {
    pub total_x_commision: u64,
    pub total_y_commision: u64,
}

impl TotalCommision {
    pub fn get_total_pubkey_with_bump() -> (Pubkey, u8) {
        Pubkey::find_program_address(&[&id().to_bytes(), POOL_SEED.as_bytes()], &id())
    }

    pub fn get_total_pubkey() -> Pubkey {
        let (pubkey, _) = Self::get_total_pubkey_with_bump();
        pubkey
    }
}
