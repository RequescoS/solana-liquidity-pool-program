use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke_signed;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::{rent::Rent, Sysvar};
use solana_program::{msg, program::invoke, program_pack::Pack, system_instruction};

use crate::error::PoolError;
use crate::instruction::PoolInstruction;
use crate::state::{TotalCommision, WithdrawedFee};
use crate::{id, POOL_SEED};

use spl_token::state::{Account, Mint};

pub const COMMISION_PERCENT: u64 = 3;
pub const SLIPPAGE_TOLERANCE: u64 = 1;

pub struct Processor;

impl Processor {
    pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = PoolInstruction::try_from_slice(input)?;
        match instruction {
            PoolInstruction::ProvideLiquidity { x_amount, y_amount } => {
                Self::provide_liquidity(accounts, x_amount, y_amount)
            }
            PoolInstruction::SwapTokens { amount } => Self::swap_tokens(accounts, amount),
            PoolInstruction::WithdrawLiquidity { amount } => {
                Self::withdraw_liquidity(accounts, amount)
            }
            PoolInstruction::WithdrawFee => Self::withdraw_fee(accounts),
        }
    }

    fn provide_liquidity(accounts: &[AccountInfo], x_amount: u64, y_amount: u64) -> ProgramResult {
        msg!("Providing liquidity");

        let acc_iter = &mut accounts.iter();
        let user_info = next_account_info(acc_iter)?;
        let withdraw_info = next_account_info(acc_iter)?;
        let x_user_token_info = next_account_info(acc_iter)?;
        let y_user_token_info = next_account_info(acc_iter)?;
        let xy_lp_user_info = next_account_info(acc_iter)?;
        let pool_x_token_info = next_account_info(acc_iter)?;
        let pool_y_token_info = next_account_info(acc_iter)?;
        let mint_lp_token_info = next_account_info(acc_iter)?;
        let current_comission_x_tokem_info = next_account_info(acc_iter)?;
        let current_comission_y_tokem_info = next_account_info(acc_iter)?;
        let total_commision_info = next_account_info(acc_iter)?;
        let admin_info = next_account_info(acc_iter)?;
        let token_info = next_account_info(acc_iter)?;
        let _ = next_account_info(acc_iter)?;
        let _ = next_account_info(acc_iter)?;

        if !user_info.is_signer {
            return Err(PoolError::SignedRequired.into());
        }

        if x_amount == 0 || y_amount == 0 {
            return Err(PoolError::ZeroProvide.into());
        }

        let x_user_token = Account::unpack_from_slice(&x_user_token_info.data.borrow())?.amount;
        let y_user_token = Account::unpack_from_slice(&y_user_token_info.data.borrow())?.amount;

        if x_amount > x_user_token || y_amount > y_user_token {
            return Err(PoolError::OverProvide.into());
        }

        Self::withdraw_fee(accounts)?;

        let ix = spl_token::instruction::transfer(
            token_info.key,
            x_user_token_info.key,
            pool_x_token_info.key,
            user_info.key,
            &[user_info.key],
            x_amount,
        )?;
        invoke(
            &ix,
            &[
                x_user_token_info.clone(),
                pool_x_token_info.clone(),
                user_info.clone(),
                token_info.clone(),
            ],
        )?;

        let iy = spl_token::instruction::transfer(
            token_info.key,
            y_user_token_info.key,
            pool_y_token_info.key,
            user_info.key,
            &[user_info.key],
            y_amount,
        )?;
        invoke(
            &iy,
            &[
                y_user_token_info.clone(),
                pool_y_token_info.clone(),
                user_info.clone(),
                token_info.clone(),
            ],
        )?;

        let total_lp = Mint::unpack_from_slice(&mint_lp_token_info.data.borrow())?;
        let pool_x_token = Account::unpack_from_slice(&pool_x_token_info.data.borrow())?;
        let pool_y_token = Account::unpack_from_slice(&pool_y_token_info.data.borrow())?;

        let new_lp: u64 = if total_lp.supply == 0 {
            ((x_amount as f64) * (y_amount as f64)).sqrt() as u64
        } else {
            if !Self::slippage_tolerance_check(pool_x_token, pool_y_token, x_amount, y_amount) {
                return Err(PoolError::SlippageFail.into());
            }
            std::cmp::min(
                x_amount * total_lp.supply / (pool_x_token.amount - x_amount),
                y_amount * total_lp.supply / (pool_y_token.amount - y_amount),
            )
        };

        msg!("mint!!!!!!!!!!!!!!!!!!!!!!");

        let ilp = spl_token::instruction::mint_to(
            token_info.key,
            mint_lp_token_info.key,
            xy_lp_user_info.key,
            admin_info.key,
            &[admin_info.key],
            new_lp,
        )?;
        invoke(
            &ilp,
            &[
                mint_lp_token_info.clone(),
                xy_lp_user_info.clone(),
                admin_info.clone(),
                token_info.clone(),
            ],
        )?;

        let total_commision = TotalCommision::try_from_slice(&total_commision_info.data.borrow())?;
        let token_x_commision =
            Account::unpack_from_slice(&current_comission_x_tokem_info.data.borrow())?.amount;
        let token_y_commision =
            Account::unpack_from_slice(&current_comission_y_tokem_info.data.borrow())?.amount;
        let total_lp = Mint::unpack_from_slice(&mint_lp_token_info.data.borrow())?.supply;
        let user_lp = Account::unpack_from_slice(&xy_lp_user_info.data.borrow())?.amount;

        let [x_amount, y_amount] = Self::liquidity_profit(
            user_lp,
            total_lp,
            token_x_commision + total_commision.total_x_commision,
            token_y_commision + total_commision.total_y_commision,
        );

        let mut withdraw = WithdrawedFee::try_from_slice(&withdraw_info.data.borrow())?;

        withdraw.user_x_withdraw = x_amount;
        withdraw.user_y_withdraw = y_amount;

        let _ = withdraw.serialize(&mut &mut withdraw_info.data.borrow_mut()[..]);

        Ok(())
    }

    pub fn slippage_tolerance_check(
        pool_x_token: Account,
        pool_y_token: Account,
        x_amount: u64,
        y_amount: u64,
    ) -> bool {
        let start_ratio: f64 =
            (pool_x_token.amount - x_amount) as f64 / (pool_y_token.amount - y_amount) as f64;
        let new_ratio: f64 = pool_x_token.amount as f64 / pool_y_token.amount as f64;
        let slippage = 1.0 - new_ratio / start_ratio;
        if slippage.abs() > SLIPPAGE_TOLERANCE as f64 / 100.0 {
            false
        } else {
            true
        }
    }

    pub fn swap_tokens(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        msg!("Swap tokens");

        let acc_iter = &mut accounts.iter();
        let user_info = next_account_info(acc_iter)?;
        let user_from_token_info = next_account_info(acc_iter)?;
        let user_to_token_info = next_account_info(acc_iter)?;
        let pool_from_token_info = next_account_info(acc_iter)?;
        let pool_to_token_info = next_account_info(acc_iter)?;
        let commision_info = next_account_info(acc_iter)?;
        let admin_info = next_account_info(acc_iter)?;
        let token_info = next_account_info(acc_iter)?;

        if !user_info.is_signer {
            return Err(PoolError::SignedRequired.into());
        }

        let pool_from_token = Account::unpack_from_slice(&pool_from_token_info.data.borrow())?;
        let pool_to_token = Account::unpack_from_slice(&pool_to_token_info.data.borrow())?;

        if amount >= pool_to_token.amount {
            return Err(PoolError::OverBuy.into());
        }

        let swap_price = Self::swap_price_define(amount, pool_from_token, pool_to_token);
        let commision_amount: u64 = swap_price * COMMISION_PERCENT / 1000;
        let user_from_token = Account::unpack_from_slice(&user_from_token_info.data.borrow())?;

        if swap_price > user_from_token.amount {
            return Err(PoolError::TooMuchBuy.into());
        }

        let buy = spl_token::instruction::transfer(
            token_info.key,
            pool_to_token_info.key,
            user_to_token_info.key,
            admin_info.key,
            &[admin_info.key],
            amount,
        )?;
        let pay = spl_token::instruction::transfer(
            token_info.key,
            user_from_token_info.key,
            pool_from_token_info.key,
            user_info.key,
            &[user_info.key],
            swap_price,
        )?;
        let comm = spl_token::instruction::transfer(
            token_info.key,
            user_from_token_info.key,
            commision_info.key,
            user_info.key,
            &[user_info.key],
            commision_amount,
        )?;
        invoke(
            &buy,
            &[
                pool_to_token_info.clone(),
                user_to_token_info.clone(),
                admin_info.clone(),
                token_info.clone(),
            ],
        )?;
        invoke(
            &pay,
            &[
                user_from_token_info.clone(),
                pool_from_token_info.clone(),
                user_info.clone(),
                token_info.clone(),
            ],
        )?;
        invoke(
            &comm,
            &[
                user_from_token_info.clone(),
                commision_info.clone(),
                user_info.clone(),
                token_info.clone(),
            ],
        )?;

        Ok(())
    }

    pub fn swap_price_define(amount: u64, pool_from_token: Account, pool_to_token: Account) -> u64 {
        ((amount as f64 / (pool_to_token.amount - amount) as f64) * pool_from_token.amount as f64)
            as u64
    }

    pub fn withdraw_liquidity(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        msg!("Withdraw liquidity");

        let acc_iter = &mut accounts.iter();
        let user_info = next_account_info(acc_iter)?;
        let _ = next_account_info(acc_iter)?;
        let x_user_token_info = next_account_info(acc_iter)?;
        let y_user_token_info = next_account_info(acc_iter)?;
        let xy_lp_user_info = next_account_info(acc_iter)?;
        let pool_x_token_info = next_account_info(acc_iter)?;
        let pool_y_token_info = next_account_info(acc_iter)?;
        let mint_lp_token_info = next_account_info(acc_iter)?;
        let _ = next_account_info(acc_iter)?;
        let _ = next_account_info(acc_iter)?;
        let _ = next_account_info(acc_iter)?;
        let admin_info = next_account_info(acc_iter)?;
        let token_info = next_account_info(acc_iter)?;
        let _ = next_account_info(acc_iter)?;
        let _ = next_account_info(acc_iter)?;

        if !user_info.is_signer {
            return Err(PoolError::SignedRequired.into());
        }

        let xy_lp_user = Account::unpack_from_slice(&xy_lp_user_info.data.borrow())?.amount;

        if amount > xy_lp_user {
            return Err(PoolError::OverWithdraw.into());
        }

        let token_x_in_pool = Account::unpack_from_slice(&pool_x_token_info.data.borrow())?.amount;
        let token_y_in_pool = Account::unpack_from_slice(&pool_y_token_info.data.borrow())?.amount;
        let total_lp = Mint::unpack_from_slice(&mint_lp_token_info.data.borrow())?.supply;
        let [x_amount, y_amount] =
            Self::liquidity_profit(amount, total_lp, token_x_in_pool, token_y_in_pool);

        Self::withdraw_fee(accounts)?;

        let user_withdraw = spl_token::instruction::burn(
            token_info.key,
            xy_lp_user_info.key,
            mint_lp_token_info.key,
            user_info.key,
            &[user_info.key],
            amount,
        )?;

        invoke(
            &user_withdraw,
            &[
                mint_lp_token_info.clone(),
                xy_lp_user_info.clone(),
                user_info.clone(),
                token_info.clone(),
            ],
        )?;

        let ix = spl_token::instruction::transfer(
            token_info.key,
            pool_x_token_info.key,
            x_user_token_info.key,
            admin_info.key,
            &[admin_info.key],
            x_amount,
        )?;
        invoke(
            &ix,
            &[
                x_user_token_info.clone(),
                pool_x_token_info.clone(),
                admin_info.clone(),
                token_info.clone(),
            ],
        )?;

        let iy = spl_token::instruction::transfer(
            token_info.key,
            pool_y_token_info.key,
            y_user_token_info.key,
            admin_info.key,
            &[admin_info.key],
            y_amount,
        )?;
        invoke(
            &iy,
            &[
                y_user_token_info.clone(),
                pool_y_token_info.clone(),
                admin_info.clone(),
                token_info.clone(),
            ],
        )?;

        Ok(())
    }

    pub fn liquidity_profit(
        amount: u64,
        total_lp: u64,
        token_x_in_pool: u64,
        token_y_in_pool: u64,
    ) -> [u64; 2] {
        if total_lp == 0 {
            return [0, 0];
        }
        let ratio = amount as f64 / total_lp as f64;
        [
            (token_x_in_pool as f64 * ratio) as u64,
            (token_y_in_pool as f64 * ratio) as u64,
        ]
    }

    pub fn withdraw_fee(accounts: &[AccountInfo]) -> ProgramResult {
        msg!("Withdraw commision");
        
        let acc_iter = &mut accounts.iter();
        let user_info = next_account_info(acc_iter)?;
        let withdraw_info = next_account_info(acc_iter)?;
        let x_user_token_info = next_account_info(acc_iter)?;
        let y_user_token_info = next_account_info(acc_iter)?;
        let xy_lp_user_info = next_account_info(acc_iter)?;
        let _ = next_account_info(acc_iter)?;
        let _ = next_account_info(acc_iter)?;
        let mint_lp_token_info = next_account_info(acc_iter)?;
        let current_comission_x_tokem_info = next_account_info(acc_iter)?;
        let current_comission_y_tokem_info = next_account_info(acc_iter)?;
        let total_commision_info = next_account_info(acc_iter)?;
        let admin_info = next_account_info(acc_iter)?;
        let token_info = next_account_info(acc_iter)?;
        let rent_info = next_account_info(acc_iter)?;
        let system_program_info = next_account_info(acc_iter)?;

        if !user_info.is_signer {
            return Err(PoolError::SignedRequired.into());
        }

        let (withdraw_pubkey, bump_seed) =
            WithdrawedFee::get_withdraw_pubkey_with_bump(user_info.key);

        if withdraw_pubkey != *withdraw_info.key {
            return Err(PoolError::WrongWithdraw.into());
        }

        if withdraw_info.data_is_empty() {
            msg!("creating new withdraw");
            let withdraw = WithdrawedFee {
                user_x_withdraw: 0,
                user_y_withdraw: 0,
            };
            let space = withdraw.try_to_vec()?.len();
            let rent = &Rent::from_account_info(rent_info)?;
            let lamports = rent.minimum_balance(space);
            let signer_seeds: &[&[_]] = &[
                &user_info.key.to_bytes(),
                POOL_SEED.as_bytes(),
                &[bump_seed],
            ];
            invoke_signed(
                &system_instruction::create_account(
                    user_info.key,
                    &withdraw_pubkey,
                    lamports,
                    space as u64,
                    &id(),
                ),
                &[
                    user_info.clone(),
                    withdraw_info.clone(),
                    system_program_info.clone(),
                ],
                &[signer_seeds],
            )?;
            let _ = withdraw.serialize(&mut &mut withdraw_info.data.borrow_mut()[..]);
        }

        let (total_commision_pubkey, bump_seed) = TotalCommision::get_total_pubkey_with_bump();

        if total_commision_pubkey != *total_commision_info.key {
            return Err(PoolError::WrongWithdraw.into());
        }

        if total_commision_info.data_is_empty() {
            msg!("creating total commision");
            let total = TotalCommision {
                total_x_commision: 0,
                total_y_commision: 0,
            };
            let space = total.try_to_vec()?.len();
            let rent = &Rent::from_account_info(rent_info)?;
            let lamports = rent.minimum_balance(space);
            let signer_seeds: &[&[_]] = &[&id().to_bytes(), POOL_SEED.as_bytes(), &[bump_seed]];
            invoke_signed(
                &system_instruction::create_account(
                    user_info.key,
                    &total_commision_pubkey,
                    lamports,
                    space as u64,
                    &id(),
                ),
                &[
                    user_info.clone(),
                    total_commision_info.clone(),
                    system_program_info.clone(),
                ],
                &[signer_seeds],
            )?;
            let _ = total.serialize(&mut &mut total_commision_info.data.borrow_mut()[..]);
        }

        let mut total_commision =
            TotalCommision::try_from_slice(&total_commision_info.data.borrow())?;

        let token_x_commision =
            Account::unpack_from_slice(&current_comission_x_tokem_info.data.borrow())?.amount;
        let token_y_commision =
            Account::unpack_from_slice(&current_comission_y_tokem_info.data.borrow())?.amount;
        let total_lp = Mint::unpack_from_slice(&mint_lp_token_info.data.borrow())?.supply;
        let user_lp = Account::unpack_from_slice(&xy_lp_user_info.data.borrow())?.amount;

        let [x_amount, y_amount] = Self::liquidity_profit(
            user_lp,
            total_lp,
            token_x_commision + total_commision.total_x_commision,
            token_y_commision + total_commision.total_y_commision,
        );

        let mut withdraw = WithdrawedFee::try_from_slice(&withdraw_info.data.borrow())?;

        let [x_amount, y_amount] = [
            x_amount - withdraw.user_x_withdraw,
            y_amount - withdraw.user_y_withdraw,
        ];

        withdraw.user_x_withdraw += x_amount;
        withdraw.user_y_withdraw += y_amount;
        total_commision.total_x_commision += x_amount;
        total_commision.total_y_commision += y_amount;

        let _ = withdraw.serialize(&mut &mut withdraw_info.data.borrow_mut()[..]);
        let _ = total_commision.serialize(&mut &mut total_commision_info.data.borrow_mut()[..]);

        let ix = spl_token::instruction::transfer(
            token_info.key,
            current_comission_x_tokem_info.key,
            x_user_token_info.key,
            admin_info.key,
            &[admin_info.key],
            x_amount,
        )?;
        invoke(
            &ix,
            &[
                x_user_token_info.clone(),
                current_comission_x_tokem_info.clone(),
                admin_info.clone(),
                token_info.clone(),
            ],
        )?;

        let iy = spl_token::instruction::transfer(
            token_info.key,
            current_comission_y_tokem_info.key,
            y_user_token_info.key,
            admin_info.key,
            &[admin_info.key],
            y_amount,
        )?;
        invoke(
            &iy,
            &[
                y_user_token_info.clone(),
                current_comission_y_tokem_info.clone(),
                admin_info.clone(),
                token_info.clone(),
            ],
        )?;

        Ok(())
    }
}
