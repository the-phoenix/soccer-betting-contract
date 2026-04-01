use cosmwasm_std::{Addr, Coin, Empty, Uint64, coins};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use crate::{
    contract::{execute, instantiate, query},
    msg::{
        BettorResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, MarketResponse, MarketStatus,
        Outcome, QueryMsg,
    },
};

fn contract_box() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new(execute, instantiate, query))
}

fn setup_app() -> App {
    let mut app = App::default();
    for addr in ["admin", "alice", "bob", "carol", "oracle"] {
        app.sudo(cw_multi_test::SudoMsg::Bank(
            cw_multi_test::BankSudo::Mint {
                to_address: addr.to_string(),
                amount: coins(1_000_000, "ucosm"),
            },
        ))
        .unwrap();
    }
    app
}

fn instantiate_contract(app: &mut App) -> Addr {
    let code_id = app.store_code(contract_box());
    app.instantiate_contract(
        code_id,
        Addr::unchecked("admin"),
        &InstantiateMsg {
            admin: None,
            treasury_bps: 250,
            stake_denom: "ucosm".into(),
        },
        &[],
        "soccer-betting",
        None,
    )
    .unwrap()
}

fn create_market(app: &mut App, contract_addr: &Addr) {
    let now = app.block_info().time.seconds();
    app.execute_contract(
        Addr::unchecked("admin"),
        contract_addr.clone(),
        &ExecuteMsg::CreateMarket {
            league: "Premier League".into(),
            home_team: "Arsenal".into(),
            away_team: "Liverpool".into(),
            kickoff_ts: Uint64::new(now + 2_000),
            close_ts: Uint64::new(now + 1_900),
            oracle: "oracle".into(),
        },
        &[],
    )
    .unwrap();
}

#[test]
fn create_bet_settle_and_claim_flow() {
    let mut app = setup_app();
    let contract_addr = instantiate_contract(&mut app);
    create_market(&mut app, &contract_addr);

    app.update_block(|block| block.time = block.time.plus_seconds(1_800));

    app.execute_contract(
        Addr::unchecked("alice"),
        contract_addr.clone(),
        &ExecuteMsg::PlaceBet {
            market_id: 1,
            outcome: Outcome::HomeWin,
        },
        &coins(100, "ucosm"),
    )
    .unwrap();
    app.execute_contract(
        Addr::unchecked("bob"),
        contract_addr.clone(),
        &ExecuteMsg::PlaceBet {
            market_id: 1,
            outcome: Outcome::HomeWin,
        },
        &coins(200, "ucosm"),
    )
    .unwrap();
    app.execute_contract(
        Addr::unchecked("carol"),
        contract_addr.clone(),
        &ExecuteMsg::PlaceBet {
            market_id: 1,
            outcome: Outcome::Draw,
        },
        &coins(101, "ucosm"),
    )
    .unwrap();

    app.update_block(|block| block.time = block.time.plus_seconds(400));

    app.execute_contract(
        Addr::unchecked("oracle"),
        contract_addr.clone(),
        &ExecuteMsg::SettleMarket {
            market_id: 1,
            outcome: Outcome::HomeWin,
        },
        &[],
    )
    .unwrap();

    let market: MarketResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &QueryMsg::Market { market_id: 1 })
        .unwrap();
    assert_eq!(market.total_staked.u128(), 401);
    assert_eq!(market.total_fee.u128(), 10);
    assert_eq!(market.total_payout_pool.u128(), 391);

    let alice_before = app.wrap().query_balance("alice", "ucosm").unwrap();
    app.execute_contract(
        Addr::unchecked("alice"),
        contract_addr.clone(),
        &ExecuteMsg::Claim { market_id: 1 },
        &[],
    )
    .unwrap();
    let alice_after = app.wrap().query_balance("alice", "ucosm").unwrap();
    assert_eq!(alice_after.amount.u128() - alice_before.amount.u128(), 130);

    let bob_before = app.wrap().query_balance("bob", "ucosm").unwrap();
    app.execute_contract(
        Addr::unchecked("bob"),
        contract_addr.clone(),
        &ExecuteMsg::Claim { market_id: 1 },
        &[],
    )
    .unwrap();
    let bob_after = app.wrap().query_balance("bob", "ucosm").unwrap();
    assert_eq!(bob_after.amount.u128() - bob_before.amount.u128(), 261);

    let bettor: BettorResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::Bettor {
                market_id: 1,
                bettor: "bob".into(),
            },
        )
        .unwrap();
    assert!(bettor.claimed);
}

#[test]
fn cancelled_market_allows_refunds() {
    let mut app = setup_app();
    let contract_addr = instantiate_contract(&mut app);
    create_market(&mut app, &contract_addr);

    app.update_block(|block| block.time = block.time.plus_seconds(1_800));
    app.execute_contract(
        Addr::unchecked("alice"),
        contract_addr.clone(),
        &ExecuteMsg::PlaceBet {
            market_id: 1,
            outcome: Outcome::HomeWin,
        },
        &coins(75, "ucosm"),
    )
    .unwrap();
    app.execute_contract(
        Addr::unchecked("alice"),
        contract_addr.clone(),
        &ExecuteMsg::PlaceBet {
            market_id: 1,
            outcome: Outcome::Draw,
        },
        &coins(25, "ucosm"),
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("admin"),
        contract_addr.clone(),
        &ExecuteMsg::CancelMarket { market_id: 1 },
        &[],
    )
    .unwrap();

    let market: MarketResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &QueryMsg::Market { market_id: 1 })
        .unwrap();
    assert!(matches!(market.status, MarketStatus::Cancelled));

    let before = app.wrap().query_balance("alice", "ucosm").unwrap();
    app.execute_contract(
        Addr::unchecked("alice"),
        contract_addr.clone(),
        &ExecuteMsg::Refund { market_id: 1 },
        &[],
    )
    .unwrap();
    let after = app.wrap().query_balance("alice", "ucosm").unwrap();
    assert_eq!(after.amount.u128() - before.amount.u128(), 100);

    let bettor: BettorResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::Bettor {
                market_id: 1,
                bettor: "alice".into(),
            },
        )
        .unwrap();
    assert!(bettor.refunded);
}

#[test]
fn rejects_late_bets() {
    let mut app = setup_app();
    let contract_addr = instantiate_contract(&mut app);
    create_market(&mut app, &contract_addr);

    app.update_block(|block| block.time = block.time.plus_seconds(1_950));
    let err = app
        .execute_contract(
            Addr::unchecked("alice"),
            contract_addr,
            &ExecuteMsg::PlaceBet {
                market_id: 1,
                outcome: Outcome::Draw,
            },
            &coins(50, "ucosm"),
        )
        .unwrap_err();

    assert!(format!("{err:?}").contains("betting is closed"));
}

#[test]
fn rejects_unauthorized_settlement() {
    let mut app = setup_app();
    let contract_addr = instantiate_contract(&mut app);
    create_market(&mut app, &contract_addr);

    app.update_block(|block| block.time = block.time.plus_seconds(2_100));
    let err = app
        .execute_contract(
            Addr::unchecked("alice"),
            contract_addr,
            &ExecuteMsg::SettleMarket {
                market_id: 1,
                outcome: Outcome::AwayWin,
            },
            &[],
        )
        .unwrap_err();

    assert!(format!("{err:?}").contains("unauthorized"));
}

#[test]
fn admin_can_withdraw_fees() {
    let mut app = setup_app();
    let contract_addr = instantiate_contract(&mut app);
    create_market(&mut app, &contract_addr);

    app.update_block(|block| block.time = block.time.plus_seconds(1_800));
    app.execute_contract(
        Addr::unchecked("alice"),
        contract_addr.clone(),
        &ExecuteMsg::PlaceBet {
            market_id: 1,
            outcome: Outcome::Draw,
        },
        &coins(200, "ucosm"),
    )
    .unwrap();

    app.update_block(|block| block.time = block.time.plus_seconds(300));
    app.execute_contract(
        Addr::unchecked("oracle"),
        contract_addr.clone(),
        &ExecuteMsg::SettleMarket {
            market_id: 1,
            outcome: Outcome::Draw,
        },
        &[],
    )
    .unwrap();

    let config: ConfigResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &QueryMsg::Config {})
        .unwrap();
    assert_eq!(config.accrued_fees.u128(), 5);

    let admin_before = app.wrap().query_balance("admin", "ucosm").unwrap();
    app.execute_contract(
        Addr::unchecked("admin"),
        contract_addr,
        &ExecuteMsg::WithdrawFees {},
        &[],
    )
    .unwrap();
    let admin_after = app.wrap().query_balance("admin", "ucosm").unwrap();
    assert_eq!(admin_after.amount.u128() - admin_before.amount.u128(), 5);
}

#[test]
fn rejects_wrong_denom() {
    let mut app = setup_app();
    let contract_addr = instantiate_contract(&mut app);
    create_market(&mut app, &contract_addr);

    app.update_block(|block| block.time = block.time.plus_seconds(1_800));
    let err = app
        .execute_contract(
            Addr::unchecked("alice"),
            contract_addr.clone(),
            &ExecuteMsg::PlaceBet {
                market_id: 1,
                outcome: Outcome::Draw,
            },
            &[Coin::new(25, "uatom")],
        )
        .unwrap_err();

    let bettor: BettorResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::Bettor {
                market_id: 1,
                bettor: "alice".into(),
            },
        )
        .unwrap();
    assert!(format!("{err:?}").contains("Error executing WasmMsg"));
    assert_eq!(bettor.draw_stake.u128(), 0);
    assert!(!bettor.claimed);
}

#[test]
fn cancelled_market_rejects_new_bets_and_settlement() {
    let mut app = setup_app();
    let contract_addr = instantiate_contract(&mut app);
    create_market(&mut app, &contract_addr);

    app.execute_contract(
        Addr::unchecked("admin"),
        contract_addr.clone(),
        &ExecuteMsg::CancelMarket { market_id: 1 },
        &[],
    )
    .unwrap();

    let bet_err = app
        .execute_contract(
            Addr::unchecked("alice"),
            contract_addr.clone(),
            &ExecuteMsg::PlaceBet {
                market_id: 1,
                outcome: Outcome::Draw,
            },
            &coins(20, "ucosm"),
        )
        .unwrap_err();
    assert!(format!("{bet_err:?}").contains("betting is closed"));

    app.update_block(|block| block.time = block.time.plus_seconds(2_100));
    let settle_err = app
        .execute_contract(
            Addr::unchecked("oracle"),
            contract_addr,
            &ExecuteMsg::SettleMarket {
                market_id: 1,
                outcome: Outcome::Draw,
            },
            &[],
        )
        .unwrap_err();
    assert!(format!("{settle_err:?}").contains("already cancelled"));
}
