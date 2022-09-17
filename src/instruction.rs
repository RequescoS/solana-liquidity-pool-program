use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

use spl_token;

use crate::{
    id,
    state::{TotalCommision, WithdrawedFee},
};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum PoolInstruction {
    /// Provide liquidity.
    /// Accounts:
    /// 0. `[signer]` user`s account
    /// 1. `[]` user`s token x account
    /// 2. `[]` user`s token y account
    /// 3. `[]` user`s token lp account
    /// 4. `[]` pool`s token x account
    /// 5. `[]` pool`s token y account
    /// 6. `[]` mint lp token account
    /// 7. `[signer]` minter account
    /// 8. `[]` token program account, PDA
    ProvideLiquidity { x_amount: u64, y_amount: u64 },

    /// Swap tokens.
    /// Accounts:
    /// 0. `[signer]` user`s account
    /// 1. `[]` user`s token from swap account
    /// 2. `[]` user`s token to swap account
    /// 3. `[]` pool`s token from swap account
    /// 4. `[]` pool`s token to swap account
    /// 5. `[]` commision from account
    /// 6. `[signer]` minter account
    /// 7. `[]` token program account, PDA
    SwapTokens { amount: u64 },

    /// Withdraw liquidity.
    /// Accounts:
    /// 0. `[signer]` user`s account
    /// 1. `[]` user`s token x account
    /// 2. `[]` user`s token y account
    /// 3. `[]` user`s token lp account
    /// 4. `[]` pool`s token x account
    /// 5. `[]` pool`s token y account
    /// 6. `[]` mint lp token account
    /// 7. `[signer]` minter account
    /// 8. `[]` token program account, PDA
    WithdrawLiquidity { amount: u64 },

    /// Withdraw fee.
    /// Accounts:
    /// 0. `[signer]` user`s account
    /// 1. '[]' user`s withdraw info account
    /// 2. `[]` user`s token x account
    /// 3. `[]` user`s token y account
    /// 4. `[]` user`s token lp account
    /// 5. `[]` mint lp token account
    /// 6. '[]' commision token x account
    /// 7. '[]' commision token y account
    /// 8. `[signer]` minter account
    /// 9. `[]` token program account, PDA
    /// 10. `[]` Rent sysvar, PDA
    /// 11. `[]` System program, PDA
    WithdrawFee,
}

impl PoolInstruction {
    pub fn provide_liquidity(
        user: &Pubkey,
        admin: &Pubkey,
        x_user_token: &Pubkey,
        y_user_token: &Pubkey,
        lp_user_token: &Pubkey,
        pool_x_token: &Pubkey,
        pool_y_token: &Pubkey,
        mint_lp_token: &Pubkey,
        commision_x_token: &Pubkey,
        commision_y_token: &Pubkey,
        x_amount: u64,
        y_amount: u64,
    ) -> Instruction {
        let withdraw_pubkey = WithdrawedFee::get_withdraw_pubkey(user);
        let total_pubkey = TotalCommision::get_total_pubkey();
        Instruction::new_with_borsh(
            id(),
            &PoolInstruction::ProvideLiquidity { x_amount, y_amount },
            vec![
                AccountMeta::new_readonly(*user, true),
                AccountMeta::new(withdraw_pubkey, false),
                AccountMeta::new(*x_user_token, false),
                AccountMeta::new(*y_user_token, false),
                AccountMeta::new(*lp_user_token, false),
                AccountMeta::new(*pool_x_token, false),
                AccountMeta::new(*pool_y_token, false),
                AccountMeta::new(*mint_lp_token, false),
                AccountMeta::new(*commision_x_token, false),
                AccountMeta::new(*commision_y_token, false),
                AccountMeta::new(total_pubkey, false),
                AccountMeta::new_readonly(*admin, true),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )
    }

    pub fn withdraw_liquidity(
        user: &Pubkey,
        admin: &Pubkey,
        x_user_token: &Pubkey,
        y_user_token: &Pubkey,
        lp_user_token: &Pubkey,
        pool_x_token: &Pubkey,
        pool_y_token: &Pubkey,
        mint_lp_token: &Pubkey,
        commision_x_token: &Pubkey,
        commision_y_token: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let withdraw_pubkey = WithdrawedFee::get_withdraw_pubkey(user);
        let total_pubkey = TotalCommision::get_total_pubkey();
        Instruction::new_with_borsh(
            id(),
            &PoolInstruction::WithdrawLiquidity { amount },
            vec![
                AccountMeta::new_readonly(*user, true),
                AccountMeta::new(withdraw_pubkey, false),
                AccountMeta::new(*x_user_token, false),
                AccountMeta::new(*y_user_token, false),
                AccountMeta::new(*lp_user_token, false),
                AccountMeta::new(*pool_x_token, false),
                AccountMeta::new(*pool_y_token, false),
                AccountMeta::new(*mint_lp_token, false),
                AccountMeta::new(*commision_x_token, false),
                AccountMeta::new(*commision_y_token, false),
                AccountMeta::new(total_pubkey, false),
                AccountMeta::new_readonly(*admin, true),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )
    }

    pub fn swap_tokens(
        user: &Pubkey,
        admin: &Pubkey,
        from_user_token: &Pubkey,
        to_user_token: &Pubkey,
        pool_from_token: &Pubkey,
        pool_to_token: &Pubkey,
        commision_from_token: &Pubkey,
        amount: u64,
    ) -> Instruction {
        Instruction::new_with_borsh(
            id(),
            &PoolInstruction::SwapTokens { amount },
            vec![
                AccountMeta::new_readonly(*user, true),
                AccountMeta::new(*from_user_token, false),
                AccountMeta::new(*to_user_token, false),
                AccountMeta::new(*pool_from_token, false),
                AccountMeta::new(*pool_to_token, false),
                AccountMeta::new(*commision_from_token, false),
                AccountMeta::new_readonly(*admin, true),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
        )
    }

    pub fn withdraw_fee(
        user: &Pubkey,
        admin: &Pubkey,
        x_user_token: &Pubkey,
        y_user_token: &Pubkey,
        lp_user_token: &Pubkey,
        pool_x_token: &Pubkey,
        pool_y_token: &Pubkey,
        mint_lp_token: &Pubkey,
        commision_x_token: &Pubkey,
        commision_y_token: &Pubkey,
    ) -> Instruction {
        let withdraw_pubkey = WithdrawedFee::get_withdraw_pubkey(user);
        let total_pubkey = TotalCommision::get_total_pubkey();
        Instruction::new_with_borsh(
            id(),
            &PoolInstruction::WithdrawFee,
            vec![
                AccountMeta::new_readonly(*user, true),
                AccountMeta::new(withdraw_pubkey, false),
                AccountMeta::new(*x_user_token, false),
                AccountMeta::new(*y_user_token, false),
                AccountMeta::new(*lp_user_token, false),
                AccountMeta::new(*pool_x_token, false),
                AccountMeta::new(*pool_y_token, false),
                AccountMeta::new(*mint_lp_token, false),
                AccountMeta::new(*commision_x_token, false),
                AccountMeta::new(*commision_y_token, false),
                AccountMeta::new(total_pubkey, false),
                AccountMeta::new_readonly(*admin, true),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )
    }
}
