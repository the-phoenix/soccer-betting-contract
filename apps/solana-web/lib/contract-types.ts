export type Outcome = "home_win" | "draw" | "away_win";
export type MarketStatus = "open" | "settled" | "cancelled";

export type ConfigResponse = {
  address: string;
  admin: string;
  treasury_bps: number;
  accrued_fees: string;
  next_market_id: number;
};

export type MarketResponse = {
  address: string;
  market_id: number;
  league: string;
  home_team: string;
  away_team: string;
  kickoff_ts: string;
  close_ts: string;
  oracle: string;
  status: MarketStatus;
  settled_outcome: Outcome | null;
  settled_at: string | null;
  total_staked: string;
  total_payout_pool: string;
  total_fee: string;
  paid_out: string;
  winning_claimed_stake: string;
  home_pool: string;
  draw_pool: string;
  away_pool: string;
};

export type BettorResponse = {
  address: string;
  bettor: string;
  market: string;
  home_stake: string;
  draw_stake: string;
  away_stake: string;
  claimed: boolean;
  refunded: boolean;
};
