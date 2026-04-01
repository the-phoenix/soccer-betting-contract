use cosmwasm_std::{
    Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint64,
    Uint128, coins, to_json_binary,
};
use cw2::set_contract_version;

use crate::{
    error::ContractError,
    msg::{
        BettorResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, MarketResponse, MarketStatus,
        Outcome, QueryMsg,
    },
    state::{BETTORS, BettorLedger, CONFIG, Config, MARKETS, Market},
};

const CONTRACT_NAME: &str = "crates.io:soccer-betting-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.treasury_bps > 10_000 {
        return Err(ContractError::InvalidFeeBps);
    }
    validate_text("stake_denom", &msg.stake_denom)?;

    let admin = match msg.admin {
        Some(admin) => deps.api.addr_validate(&admin)?,
        None => info.sender.clone(),
    };

    let config = Config {
        admin: admin.clone(),
        treasury_bps: msg.treasury_bps,
        stake_denom: msg.stake_denom.clone(),
        accrued_fees: Uint128::zero(),
        next_market_id: 1,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin)
        .add_attribute("stake_denom", msg.stake_denom)
        .add_attribute("treasury_bps", msg.treasury_bps.to_string()))
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateMarket {
            league,
            home_team,
            away_team,
            kickoff_ts,
            close_ts,
            oracle,
        } => execute_create_market(
            deps, info, league, home_team, away_team, kickoff_ts, close_ts, oracle,
        ),
        ExecuteMsg::PlaceBet { market_id, outcome } => {
            execute_place_bet(deps, env, info, market_id, outcome)
        }
        ExecuteMsg::SettleMarket { market_id, outcome } => {
            execute_settle_market(deps, env, info, market_id, outcome)
        }
        ExecuteMsg::CancelMarket { market_id } => execute_cancel_market(deps, info, market_id),
        ExecuteMsg::Claim { market_id } => execute_claim(deps, info, market_id),
        ExecuteMsg::Refund { market_id } => execute_refund(deps, info, market_id),
        ExecuteMsg::WithdrawFees {} => execute_withdraw_fees(deps, info),
    }
}

fn execute_create_market(
    deps: DepsMut,
    info: MessageInfo,
    league: String,
    home_team: String,
    away_team: String,
    kickoff_ts: Uint64,
    close_ts: Uint64,
    oracle: String,
) -> Result<Response, ContractError> {
    validate_text("league", &league)?;
    validate_text("home_team", &home_team)?;
    validate_text("away_team", &away_team)?;
    validate_text("oracle", &oracle)?;

    let mut config = CONFIG.load(deps.storage)?;
    ensure_admin(&config, &info.sender)?;

    if close_ts >= kickoff_ts {
        return Err(ContractError::InvalidSchedule);
    }

    let oracle = deps.api.addr_validate(&oracle)?;
    let market_id = config.next_market_id;
    config.next_market_id += 1;

    let market = Market {
        id: market_id,
        league,
        home_team,
        away_team,
        kickoff_ts,
        close_ts,
        oracle: oracle.clone(),
        status: MarketStatus::Open,
        settled_outcome: None,
        settled_at: None,
        total_staked: Uint128::zero(),
        total_payout_pool: Uint128::zero(),
        total_fee: Uint128::zero(),
        paid_out: Uint128::zero(),
        winning_claimed_stake: Uint128::zero(),
        pools: [Uint128::zero(), Uint128::zero(), Uint128::zero()],
    };

    CONFIG.save(deps.storage, &config)?;
    MARKETS.save(deps.storage, market_id, &market)?;

    Ok(Response::new()
        .add_attribute("action", "create_market")
        .add_attribute("market_id", market_id.to_string())
        .add_attribute("oracle", oracle))
}

fn execute_place_bet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    market_id: u64,
    outcome: Outcome,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut market = load_market(deps.storage, market_id)?;

    if !matches!(market.status, MarketStatus::Open)
        || env.block.time.seconds() >= market.close_ts.u64()
    {
        return Err(ContractError::BettingClosed { market_id });
    }

    let amount = must_pay_exact_denom(&info.funds, &config.stake_denom)?;
    if amount.is_zero() {
        return Err(ContractError::ZeroAmount);
    }

    market.total_staked = market.total_staked.checked_add(amount)?;
    market.pools[outcome.index()] = market.pools[outcome.index()].checked_add(amount)?;
    MARKETS.save(deps.storage, market_id, &market)?;

    let bettor = info.sender.clone();
    let mut ledger = BETTORS
        .may_load(deps.storage, (market_id, &bettor))?
        .unwrap_or_default();
    ledger.stakes[outcome.index()] = ledger.stakes[outcome.index()].checked_add(amount)?;
    BETTORS.save(deps.storage, (market_id, &bettor), &ledger)?;

    Ok(Response::new()
        .add_attribute("action", "place_bet")
        .add_attribute("market_id", market_id.to_string())
        .add_attribute("bettor", info.sender)
        .add_attribute("outcome", outcome_label(&outcome))
        .add_attribute("amount", amount))
}

fn execute_settle_market(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    market_id: u64,
    outcome: Outcome,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut market = load_market(deps.storage, market_id)?;

    if matches!(market.status, MarketStatus::Settled) {
        return Err(ContractError::MarketAlreadySettled { market_id });
    }
    if matches!(market.status, MarketStatus::Cancelled) {
        return Err(ContractError::MarketAlreadyCancelled { market_id });
    }
    if info.sender != config.admin && info.sender != market.oracle {
        return Err(ContractError::Unauthorized {
            sender: info.sender.clone(),
        });
    }
    if env.block.time.seconds() < market.kickoff_ts.u64() {
        return Err(ContractError::BetTooEarlyToSettle { market_id });
    }

    let fee = market
        .total_staked
        .multiply_ratio(config.treasury_bps as u128, 10_000u128);
    market.total_fee = fee;
    market.total_payout_pool = market.total_staked.checked_sub(fee)?;
    market.status = MarketStatus::Settled;
    market.settled_outcome = Some(outcome.clone());
    market.settled_at = Some(Uint64::new(env.block.time.seconds()));
    config.accrued_fees = config.accrued_fees.checked_add(fee)?;

    CONFIG.save(deps.storage, &config)?;
    MARKETS.save(deps.storage, market_id, &market)?;

    Ok(Response::new()
        .add_attribute("action", "settle_market")
        .add_attribute("market_id", market_id.to_string())
        .add_attribute("outcome", outcome_label(&outcome))
        .add_attribute("fee", fee))
}

fn execute_cancel_market(
    deps: DepsMut,
    info: MessageInfo,
    market_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    ensure_admin(&config, &info.sender)?;

    let mut market = load_market(deps.storage, market_id)?;
    if matches!(market.status, MarketStatus::Settled) {
        return Err(ContractError::MarketAlreadySettled { market_id });
    }
    if matches!(market.status, MarketStatus::Cancelled) {
        return Err(ContractError::MarketAlreadyCancelled { market_id });
    }

    market.status = MarketStatus::Cancelled;
    MARKETS.save(deps.storage, market_id, &market)?;

    Ok(Response::new()
        .add_attribute("action", "cancel_market")
        .add_attribute("market_id", market_id.to_string()))
}

fn execute_claim(
    deps: DepsMut,
    info: MessageInfo,
    market_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut market = load_market(deps.storage, market_id)?;
    if !matches!(market.status, MarketStatus::Settled) {
        return Err(ContractError::BettingClosed { market_id });
    }
    let outcome = market
        .settled_outcome
        .clone()
        .ok_or(ContractError::BettingClosed { market_id })?;
    let winning_pool = market.pools[outcome.index()];

    let bettor = info.sender.clone();
    let mut ledger = BETTORS
        .may_load(deps.storage, (market_id, &bettor))?
        .unwrap_or_default();

    if ledger.claimed {
        return Err(ContractError::AlreadyClaimed);
    }

    let winning_stake = ledger.stakes[outcome.index()];
    if winning_stake.is_zero() {
        return Err(ContractError::NoWinningBet);
    }

    let remaining_winning_stake = winning_pool.checked_sub(market.winning_claimed_stake)?;
    let remaining_payout_pool = market.total_payout_pool.checked_sub(market.paid_out)?;

    let payout = if winning_stake == remaining_winning_stake {
        remaining_payout_pool
    } else {
        market
            .total_payout_pool
            .multiply_ratio(winning_stake, winning_pool)
    };

    ledger.claimed = true;
    market.winning_claimed_stake = market.winning_claimed_stake.checked_add(winning_stake)?;
    market.paid_out = market.paid_out.checked_add(payout)?;

    BETTORS.save(deps.storage, (market_id, &bettor), &ledger)?;
    MARKETS.save(deps.storage, market_id, &market)?;

    let send = BankMsg::Send {
        to_address: bettor.to_string(),
        amount: coins(payout.u128(), config.stake_denom),
    };

    Ok(Response::new()
        .add_message(send)
        .add_attribute("action", "claim")
        .add_attribute("market_id", market_id.to_string())
        .add_attribute("bettor", bettor)
        .add_attribute("payout", payout))
}

fn execute_refund(
    deps: DepsMut,
    info: MessageInfo,
    market_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let market = load_market(deps.storage, market_id)?;
    if !matches!(market.status, MarketStatus::Cancelled) {
        return Err(ContractError::MarketNotCancelled { market_id });
    }

    let bettor = info.sender.clone();
    let mut ledger = BETTORS
        .may_load(deps.storage, (market_id, &bettor))?
        .unwrap_or_default();

    if ledger.refunded {
        return Err(ContractError::AlreadyRefunded);
    }

    let refund_amount = ledger
        .stakes
        .into_iter()
        .fold(Uint128::zero(), |acc, stake| acc + stake);
    if refund_amount.is_zero() {
        return Err(ContractError::NoWinningBet);
    }

    ledger.refunded = true;
    BETTORS.save(deps.storage, (market_id, &bettor), &ledger)?;

    let send = BankMsg::Send {
        to_address: bettor.to_string(),
        amount: coins(refund_amount.u128(), config.stake_denom),
    };

    Ok(Response::new()
        .add_message(send)
        .add_attribute("action", "refund")
        .add_attribute("market_id", market_id.to_string())
        .add_attribute("bettor", bettor)
        .add_attribute("amount", refund_amount))
}

fn execute_withdraw_fees(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    ensure_admin(&config, &info.sender)?;

    let amount = config.accrued_fees;
    config.accrued_fees = Uint128::zero();
    CONFIG.save(deps.storage, &config)?;

    let msg = BankMsg::Send {
        to_address: config.admin.to_string(),
        amount: coins(amount.u128(), config.stake_denom.clone()),
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "withdraw_fees")
        .add_attribute("amount", amount))
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Market { market_id } => to_json_binary(&query_market(deps, market_id)?),
        QueryMsg::Bettor { market_id, bettor } => {
            to_json_binary(&query_bettor(deps, market_id, bettor)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        admin: config.admin.to_string(),
        treasury_bps: config.treasury_bps,
        stake_denom: config.stake_denom,
        accrued_fees: config.accrued_fees,
        next_market_id: config.next_market_id,
    })
}

fn query_market(deps: Deps, market_id: u64) -> StdResult<MarketResponse> {
    let market = MARKETS.load(deps.storage, market_id)?;
    Ok(MarketResponse {
        market_id: market.id,
        league: market.league,
        home_team: market.home_team,
        away_team: market.away_team,
        kickoff_ts: market.kickoff_ts,
        close_ts: market.close_ts,
        oracle: market.oracle.to_string(),
        status: market.status,
        settled_outcome: market.settled_outcome,
        settled_at: market.settled_at,
        total_staked: market.total_staked,
        total_payout_pool: market.total_payout_pool,
        total_fee: market.total_fee,
        paid_out: market.paid_out,
        winning_claimed_stake: market.winning_claimed_stake,
        home_pool: market.pools[0],
        draw_pool: market.pools[1],
        away_pool: market.pools[2],
    })
}

fn query_bettor(deps: Deps, market_id: u64, bettor: String) -> StdResult<BettorResponse> {
    let bettor_addr = deps.api.addr_validate(&bettor)?;
    let ledger = BETTORS
        .may_load(deps.storage, (market_id, &bettor_addr))?
        .unwrap_or(BettorLedger {
            stakes: [Uint128::zero(), Uint128::zero(), Uint128::zero()],
            claimed: false,
            refunded: false,
        });

    Ok(BettorResponse {
        bettor,
        market_id,
        home_stake: ledger.stakes[0],
        draw_stake: ledger.stakes[1],
        away_stake: ledger.stakes[2],
        claimed: ledger.claimed,
        refunded: ledger.refunded,
    })
}

fn ensure_admin(config: &Config, sender: &Addr) -> Result<(), ContractError> {
    if sender != &config.admin {
        return Err(ContractError::Unauthorized {
            sender: sender.clone(),
        });
    }
    Ok(())
}

fn validate_text(field: &'static str, value: &str) -> Result<(), ContractError> {
    if value.trim().is_empty() {
        return Err(ContractError::EmptyField { field });
    }
    Ok(())
}

fn load_market(
    storage: &dyn cosmwasm_std::Storage,
    market_id: u64,
) -> Result<Market, ContractError> {
    MARKETS
        .may_load(storage, market_id)?
        .ok_or(ContractError::MarketNotFound { market_id })
}

fn must_pay_exact_denom(funds: &[Coin], expected: &str) -> Result<Uint128, ContractError> {
    if funds.len() != 1 {
        return Err(ContractError::InvalidFunds {
            expected: expected.to_string(),
        });
    }

    let coin = &funds[0];
    if coin.denom != expected {
        return Err(ContractError::InvalidDenom {
            expected: expected.to_string(),
        });
    }

    Ok(coin.amount)
}

fn outcome_label(outcome: &Outcome) -> &'static str {
    match outcome {
        Outcome::HomeWin => "home_win",
        Outcome::Draw => "draw",
        Outcome::AwayWin => "away_win",
    }
}
