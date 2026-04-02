use cosmwasm_schema::{QueryResponses, cw_serde};
use cosmwasm_std::{Uint64, Uint128};

#[cw_serde]
pub enum Outcome {
    HomeWin,
    Draw,
    AwayWin,
}

impl Outcome {
    pub fn index(&self) -> usize {
        match self {
            Outcome::HomeWin => 0,
            Outcome::Draw => 1,
            Outcome::AwayWin => 2,
        }
    }
}

#[cw_serde]
pub enum MarketStatus {
    Open,
    Settled,
    Cancelled,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub treasury_bps: u16,
    pub stake_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateMarket {
        league: String,
        home_team: String,
        away_team: String,
        kickoff_ts: Uint64,
        close_ts: Uint64,
        oracle: String,
    },
    PlaceBet {
        market_id: u64,
        outcome: Outcome,
    },
    SettleMarket {
        market_id: u64,
        outcome: Outcome,
    },
    CancelMarket {
        market_id: u64,
    },
    Claim {
        market_id: u64,
    },
    Refund {
        market_id: u64,
    },
    WithdrawFees {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(MarketResponse)]
    Market { market_id: u64 },
    #[returns(BettorResponse)]
    Bettor { market_id: u64, bettor: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub treasury_bps: u16,
    pub stake_denom: String,
    pub accrued_fees: Uint128,
    pub next_market_id: u64,
}

#[cw_serde]
pub struct MarketResponse {
    pub market_id: u64,
    pub league: String,
    pub home_team: String,
    pub away_team: String,
    pub kickoff_ts: Uint64,
    pub close_ts: Uint64,
    pub oracle: String,
    pub status: MarketStatus,
    pub settled_outcome: Option<Outcome>,
    pub settled_at: Option<Uint64>,
    pub total_staked: Uint128,
    pub total_payout_pool: Uint128,
    pub total_fee: Uint128,
    pub paid_out: Uint128,
    pub winning_claimed_stake: Uint128,
    pub home_pool: Uint128,
    pub draw_pool: Uint128,
    pub away_pool: Uint128,
}

#[cw_serde]
pub struct BettorResponse {
    pub bettor: String,
    pub market_id: u64,
    pub home_stake: Uint128,
    pub draw_stake: Uint128,
    pub away_stake: Uint128,
    pub claimed: bool,
    pub refunded: bool,
}
