#![cfg(feature = "test-bpf")]
use std::assert_eq;

use borsh::BorshDeserialize;
use solana_program::{program_pack::Pack, pubkey::Pubkey, system_instruction};
use solana_program_test::{
    processor,
    tokio::{self},
    ProgramTest, ProgramTestContext,
};

use spl_token::state::{Account, Mint};

use pool::{entrypoint::process_instruction, id, instruction::PoolInstruction};
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

struct Env {
    ctx: ProgramTestContext,
    admin: Keypair,
    user_01: Keypair,
    user_02: Keypair,
    mint_lp_account: Keypair,
    user_01_x_token_account: Keypair,
    user_01_y_token_account: Keypair,
    user_01_lp_token_account: Keypair,
    user_02_x_token_account: Keypair,
    user_02_y_token_account: Keypair,
    user_02_lp_token_account: Keypair,
    pool_x_token_account: Keypair,
    pool_y_token_account: Keypair,
    commision_x_token_account: Keypair,
    commision_y_token_account: Keypair,
}

impl Env {
    async fn new() -> Self {
        let program_test = ProgramTest::new("pool", id(), processor!(process_instruction));
        let mut ctx = program_test.start_with_context().await;

        let admin = Keypair::new();
        let user_01 = Keypair::new();
        let user_02 = Keypair::new();

        ctx.banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[
                    system_instruction::transfer(
                        &ctx.payer.pubkey(),
                        &admin.pubkey(),
                        1_000_000_000,
                    ),
                    system_instruction::transfer(
                        &ctx.payer.pubkey(),
                        &user_01.pubkey(),
                        1_000_000_000,
                    ),
                    system_instruction::transfer(
                        &ctx.payer.pubkey(),
                        &user_02.pubkey(),
                        1_000_000_000,
                    ),
                ],
                Some(&ctx.payer.pubkey()),
                &[&ctx.payer],
                ctx.last_blockhash,
            ))
            .await
            .unwrap();

        let mint_x_account = Keypair::new();
        let mint_y_account = Keypair::new();
        let mint_lp_account = Keypair::new();
        let mint_array = [&mint_x_account, &mint_y_account, &mint_lp_account];

        let token_program = &spl_token::id();
        let rent = ctx.banks_client.get_rent().await.unwrap();
        let mint_rent = rent.minimum_balance(Mint::LEN);

        for i in mint_array {
            let token_mint_account_ix = solana_program::system_instruction::create_account(
                &ctx.payer.pubkey(),
                &i.pubkey(),
                mint_rent,
                Mint::LEN as u64,
                token_program,
            );

            let token_mint_a_ix = spl_token::instruction::initialize_mint(
                token_program,
                &i.pubkey(),
                &admin.pubkey(),
                None,
                9,
            )
            .unwrap();

            let token_mint_a_tx = Transaction::new_signed_with_payer(
                &[token_mint_account_ix, token_mint_a_ix],
                Some(&ctx.payer.pubkey()),
                &[&ctx.payer, &i],
                ctx.last_blockhash,
            );

            ctx.banks_client
                .process_transaction(token_mint_a_tx)
                .await
                .unwrap();
        }

        let user_01_x_token_account = Keypair::new();
        let user_01_y_token_account = Keypair::new();
        let user_01_lp_token_account = Keypair::new();
        let user_02_x_token_account = Keypair::new();
        let user_02_y_token_account = Keypair::new();
        let user_02_lp_token_account = Keypair::new();
        let user_wallets = [
            [&user_01_x_token_account, &mint_x_account, &user_01],
            [&user_01_y_token_account, &mint_y_account, &user_01],
            [&user_01_lp_token_account, &mint_lp_account, &user_01],
            [&user_02_x_token_account, &mint_x_account, &user_02],
            [&user_02_y_token_account, &mint_y_account, &user_02],
            [&user_02_lp_token_account, &mint_lp_account, &user_02],
        ];
        let account_rent = rent.minimum_balance(Account::LEN);

        for [i, j, k] in user_wallets {
            let token_associated_account_ix = solana_program::system_instruction::create_account(
                &ctx.payer.pubkey(),
                &i.pubkey(),
                account_rent,
                Account::LEN as u64,
                token_program,
            );

            let initialize_account_a_ix = spl_token::instruction::initialize_account(
                token_program,
                &i.pubkey(),
                &j.pubkey(),
                &k.pubkey(),
            )
            .unwrap();

            let create_new_associated_token_account_tx = Transaction::new_signed_with_payer(
                &[token_associated_account_ix, initialize_account_a_ix],
                Some(&ctx.payer.pubkey()),
                &[&ctx.payer, &i],
                ctx.last_blockhash,
            );

            ctx.banks_client
                .process_transaction(create_new_associated_token_account_tx)
                .await
                .unwrap();
        }

        let pool_x_token_account = Keypair::new();
        let pool_y_token_account = Keypair::new();
        let pool_wallets = [
            [&pool_x_token_account, &mint_x_account],
            [&pool_y_token_account, &mint_y_account],
        ];

        for [i, j] in pool_wallets {
            let pool_token_associated_account_ix =
                solana_program::system_instruction::create_account(
                    &ctx.payer.pubkey(),
                    &i.pubkey(),
                    account_rent,
                    Account::LEN as u64,
                    token_program,
                );

            let initialize_account_a_ix = spl_token::instruction::initialize_account(
                token_program,
                &i.pubkey(),
                &j.pubkey(),
                &admin.pubkey(),
            )
            .unwrap();

            let create_new_pool_associated_token_account_tx = Transaction::new_signed_with_payer(
                &[pool_token_associated_account_ix, initialize_account_a_ix],
                Some(&ctx.payer.pubkey()),
                &[&ctx.payer, &i],
                ctx.last_blockhash,
            );

            ctx.banks_client
                .process_transaction(create_new_pool_associated_token_account_tx)
                .await
                .unwrap();
        }

        let commision_x_token_account = Keypair::new();
        let commision_y_token_account = Keypair::new();
        let commision_wallets = [
            [&commision_x_token_account, &mint_x_account],
            [&commision_y_token_account, &mint_y_account],
        ];

        for [i, j] in commision_wallets {
            let pool_token_associated_account_ix =
                solana_program::system_instruction::create_account(
                    &ctx.payer.pubkey(),
                    &i.pubkey(),
                    account_rent,
                    Account::LEN as u64,
                    token_program,
                );

            let initialize_account_a_ix = spl_token::instruction::initialize_account(
                token_program,
                &i.pubkey(),
                &j.pubkey(),
                &admin.pubkey(),
            )
            .unwrap();

            let create_new_pool_associated_token_account_tx = Transaction::new_signed_with_payer(
                &[pool_token_associated_account_ix, initialize_account_a_ix],
                Some(&ctx.payer.pubkey()),
                &[&ctx.payer, &i],
                ctx.last_blockhash,
            );

            ctx.banks_client
                .process_transaction(create_new_pool_associated_token_account_tx)
                .await
                .unwrap();
        }

        let need_to_mint = [
            [&mint_x_account, &user_01_x_token_account],
            [&mint_y_account, &user_01_y_token_account],
            [&mint_x_account, &user_02_x_token_account],
            [&mint_y_account, &user_02_y_token_account],
        ];

        for [mint_account, user_token_account] in need_to_mint {
            let mint_user_token = Transaction::new_signed_with_payer(
                &[spl_token::instruction::mint_to(
                    token_program,
                    &mint_account.pubkey(),
                    &user_token_account.pubkey(),
                    &admin.pubkey(),
                    &[&admin.pubkey()],
                    10000000,
                )
                .unwrap()],
                Some(&ctx.payer.pubkey()),
                &[&ctx.payer, &admin],
                ctx.last_blockhash,
            );

            ctx.banks_client
                .process_transaction(mint_user_token)
                .await
                .unwrap();
        }

        Env {
            ctx,
            admin,
            user_01,
            user_02,
            mint_lp_account,
            user_01_x_token_account,
            user_01_y_token_account,
            user_01_lp_token_account,
            user_02_x_token_account,
            user_02_y_token_account,
            user_02_lp_token_account,
            pool_x_token_account,
            pool_y_token_account,
            commision_x_token_account,
            commision_y_token_account,
        }
    }
}

// test of first user providing liquidity
#[tokio::test]
async fn provide_liquidity_first() {
    let mut env = Env::new().await;

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::provide_liquidity(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.user_01_lp_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.mint_lp_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            &env.commision_y_token_account.pubkey(),
            5,
            15,
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.user_01_lp_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let user_lp = Account::unpack_from_slice(&acc.data.as_slice()).unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.mint_lp_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let total_lp = Mint::unpack_from_slice(&acc.data.as_slice()).unwrap();

    assert_eq!(user_lp.amount, 8);
    assert_eq!(total_lp.supply, 8);
}

// withdraw part of user`s liquidity
#[tokio::test]
async fn part_liquidity_withdraw() {
    let mut env = Env::new().await;

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::provide_liquidity(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.user_01_lp_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.mint_lp_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            &env.commision_y_token_account.pubkey(),
            5,
            15,
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.user_01_x_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let user_x_token_start = Account::unpack_from_slice(&acc.data.as_slice()).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::withdraw_liquidity(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.user_01_lp_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.mint_lp_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            &env.commision_y_token_account.pubkey(),
            5,
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.user_01_lp_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let user_lp = Account::unpack_from_slice(&acc.data.as_slice()).unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.mint_lp_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let total_lp = Mint::unpack_from_slice(&acc.data.as_slice()).unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.user_01_x_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let user_x_token_new = Account::unpack_from_slice(&acc.data.as_slice()).unwrap();

    let withdraw_user_x_token = user_x_token_new.amount - user_x_token_start.amount;

    assert_eq!(user_lp.amount, 3);
    assert_eq!(total_lp.supply, 3);
    assert_eq!(withdraw_user_x_token, 3);
}

// test of tokens swap
#[tokio::test]
async fn swap() {
    let mut env = Env::new().await;

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::provide_liquidity(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.user_01_lp_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.mint_lp_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            &env.commision_y_token_account.pubkey(),
            5,
            15,
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.user_01_y_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let user_y_start = Account::unpack_from_slice(&acc.data.as_slice()).unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.user_01_x_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let user_x_start = Account::unpack_from_slice(&acc.data.as_slice()).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::swap_tokens(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            13,
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.user_01_y_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let user_y_new = Account::unpack_from_slice(&acc.data.as_slice()).unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.user_01_x_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let user_x_new = Account::unpack_from_slice(&acc.data.as_slice()).unwrap();

    let swap_y: i64 = user_y_new.amount as i64 - user_y_start.amount as i64;
    let swap_x: i64 = user_x_new.amount as i64 - user_x_start.amount as i64;

    assert_eq!(swap_y, 13);
    assert_eq!(swap_x, -32);
}

// user first time withdraw commision
#[tokio::test]
async fn withdraw_fee_first() {
    let mut env = Env::new().await;

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::provide_liquidity(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.user_01_lp_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.mint_lp_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            &env.commision_y_token_account.pubkey(),
            500000,
            750000,
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::swap_tokens(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            250000,
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.user_01_x_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let user_x_start = Account::unpack_from_slice(&acc.data.as_slice())
        .unwrap()
        .amount;

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::withdraw_fee(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.user_01_lp_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.mint_lp_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            &env.commision_y_token_account.pubkey(),
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.user_01_x_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let user_x_new = Account::unpack_from_slice(&acc.data.as_slice())
        .unwrap()
        .amount;

    let get_commision = user_x_new - user_x_start;

    assert_eq!(get_commision, 750);
}

// user withdraw commision second time, but new commision don`t arrived yet
#[tokio::test]
async fn withdraw_fee_second_time() {
    let mut env = Env::new().await;

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::provide_liquidity(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.user_01_lp_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.mint_lp_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            &env.commision_y_token_account.pubkey(),
            500000,
            750000,
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::swap_tokens(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            250000,
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::withdraw_fee(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.user_01_lp_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.mint_lp_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            &env.commision_y_token_account.pubkey(),
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.user_01_x_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let user_x_start = Account::unpack_from_slice(&acc.data.as_slice())
        .unwrap()
        .amount;

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::withdraw_fee(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.user_01_lp_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.mint_lp_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            &env.commision_y_token_account.pubkey(),
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.user_01_x_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let user_x_new = Account::unpack_from_slice(&acc.data.as_slice())
        .unwrap()
        .amount;

    let get_commision = user_x_new - user_x_start;

    assert_eq!(get_commision, 0);
}

// test for user not immediately get new commision fees after providing liquidity
#[tokio::test]
async fn provide_new_liquidity() {
    let mut env = Env::new().await;

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::provide_liquidity(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.user_01_lp_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.mint_lp_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            &env.commision_y_token_account.pubkey(),
            50000,
            75000,
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::swap_tokens(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            25000,
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::provide_liquidity(
            &env.user_02.pubkey(),
            &env.admin.pubkey(),
            &env.user_02_x_token_account.pubkey(),
            &env.user_02_y_token_account.pubkey(),
            &env.user_02_lp_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.mint_lp_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            &env.commision_y_token_account.pubkey(),
            75000,
            50000,
        )],
        Some(&env.user_02.pubkey()),
        &[&env.user_02, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::withdraw_fee(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.user_01_lp_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.mint_lp_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            &env.commision_y_token_account.pubkey(),
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::provide_liquidity(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.user_01_lp_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.mint_lp_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            &env.commision_y_token_account.pubkey(),
            75000,
            50000,
        )],
        Some(&env.admin.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.user_01_x_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let user_x_start = Account::unpack_from_slice(&acc.data.as_slice())
        .unwrap()
        .amount;

    let tx = Transaction::new_signed_with_payer(
        &[PoolInstruction::withdraw_fee(
            &env.user_01.pubkey(),
            &env.admin.pubkey(),
            &env.user_01_x_token_account.pubkey(),
            &env.user_01_y_token_account.pubkey(),
            &env.user_01_lp_token_account.pubkey(),
            &env.pool_x_token_account.pubkey(),
            &env.pool_y_token_account.pubkey(),
            &env.mint_lp_account.pubkey(),
            &env.commision_x_token_account.pubkey(),
            &env.commision_y_token_account.pubkey(),
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01, &env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(env.user_01_x_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let user_x_new = Account::unpack_from_slice(&acc.data.as_slice())
        .unwrap()
        .amount;

    let get_commision = user_x_new - user_x_start;

    assert_eq!(get_commision, 0);
}
