use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint64, Uint128};
use cw_storage_plus::{Item, Map};

use crate::msg::{MarketStatus, Outcome};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub treasury_bps: u16,
    pub stake_denom: String,
    pub accrued_fees: Uint128,
    pub next_market_id: u64,
}

#[cw_serde]
pub struct Market {
    pub id: u64,
    pub league: String,
    pub home_team: String,
    pub away_team: String,
    pub kickoff_ts: Uint64,
    pub close_ts: Uint64,
    pub oracle: Addr,
    pub status: MarketStatus,
    pub settled_outcome: Option<Outcome>,
    pub settled_at: Option<Uint64>,
    pub total_staked: Uint128,
    pub total_payout_pool: Uint128,
    pub total_fee: Uint128,
    pub paid_out: Uint128,
    pub winning_claimed_stake: Uint128,
    pub pools: [Uint128; 3],
}

#[cw_serde]
pub struct BettorLedger {
    pub stakes: [Uint128; 3],
    pub claimed: bool,
    pub refunded: bool,
}

impl Default for BettorLedger {
    fn default() -> Self {
        Self {
            stakes: [Uint128::zero(), Uint128::zero(), Uint128::zero()],
            claimed: false,
            refunded: false,
        }
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const MARKETS: Map<u64, Market> = Map::new("markets");
pub const BETTORS: Map<(u64, &Addr), BettorLedger> = Map::new("bettors");
