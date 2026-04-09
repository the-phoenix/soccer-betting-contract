export type Outcome = "home_win" | "draw" | "away_win";
export type MarketStatus = "open" | "settled" | "cancelled";

export type ConfigResponse = {
  admin: string;
  treasury_bps: number;
  stake_denom: string;
  accrued_fees: string;
  next_market_id: number;
};

export type MarketResponse = {
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
  bettor: string;
  market_id: number;
  home_stake: string;
  draw_stake: string;
  away_stake: string;
  claimed: boolean;
  refunded: boolean;
};

export type ExecuteMessage =
  | {
      create_market: {
        league: string;
        home_team: string;
        away_team: string;
        kickoff_ts: string;
        close_ts: string;
        oracle: string;
      };
    }
  | {
      place_bet: {
        market_id: number;
        outcome: Outcome;
      };
    }
  | {
      settle_market: {
        market_id: number;
        outcome: Outcome;
      };
    }
  | {
      cancel_market: {
        market_id: number;
      };
    }
  | {
      claim: {
        market_id: number;
      };
    }
  | {
      refund: {
        market_id: number;
      };
    }
  | {
      withdraw_fees: Record<string, never>;
    };
