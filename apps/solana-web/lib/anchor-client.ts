import { Buffer } from "buffer";
import { PublicKey, Connection } from "@solana/web3.js";

import { appConfig } from "./config";
import type {
  BettorResponse,
  ConfigResponse,
  MarketResponse,
  MarketStatus,
  Outcome,
} from "./contract-types";

const CONFIG_DISCRIMINATOR = Buffer.from("9b0caae01efacc82", "hex");
const MARKET_DISCRIMINATOR = Buffer.from("dbbed53700e3c69a", "hex");
const BETTOR_DISCRIMINATOR = Buffer.from("55ecdf60da7ca5b0", "hex");

export function getConnection() {
  if (!appConfig.rpcEndpoint) {
    throw new Error("NEXT_PUBLIC_SOLANA_RPC_ENDPOINT is not configured.");
  }
  return new Connection(appConfig.rpcEndpoint, "confirmed");
}

export function getProgramId() {
  if (!appConfig.programId) {
    throw new Error("NEXT_PUBLIC_PROGRAM_ID is not configured.");
  }
  return new PublicKey(appConfig.programId);
}

export function findConfigAddress() {
  return PublicKey.findProgramAddressSync([Buffer.from("config")], getProgramId());
}

export function findMarketAddress(marketId: number | bigint) {
  const seed = Buffer.alloc(8);
  seed.writeBigUInt64LE(BigInt(marketId));
  return PublicKey.findProgramAddressSync(
    [Buffer.from("market"), seed],
    getProgramId(),
  );
}

export function findBettorLedgerAddress(marketAddress: PublicKey, bettor: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("bettor"), marketAddress.toBuffer(), bettor.toBuffer()],
    getProgramId(),
  );
}

export async function queryConfig() {
  const connection = getConnection();
  const [configAddress] = findConfigAddress();
  const account = await connection.getAccountInfo(configAddress);

  if (!account) {
    throw new Error("Config account was not found.");
  }

  return decodeConfig(configAddress, account.data);
}

export async function queryMarket(marketId: number) {
  const connection = getConnection();
  const [marketAddress] = findMarketAddress(marketId);
  const account = await connection.getAccountInfo(marketAddress);

  if (!account) {
    throw new Error(`Market ${marketId} was not found.`);
  }

  return decodeMarket(marketAddress, account.data);
}

export async function queryMarkets(limit = appConfig.defaultMarketLimit) {
  const config = await queryConfig();
  const highestMarketId = Math.max(config.next_market_id - 1, 0);
  const marketIds = Array.from(
    { length: Math.min(limit, highestMarketId) },
    (_, index) => highestMarketId - index,
  ).filter((marketId) => marketId > 0);

  const markets = await Promise.all(
    marketIds.map(async (marketId) => {
      try {
        return await queryMarket(marketId);
      } catch {
        return null;
      }
    }),
  );

  return markets.filter((market): market is MarketResponse => market !== null);
}

export async function queryBettor(marketId: number, bettor: string) {
  const connection = getConnection();
  const bettorAddress = new PublicKey(bettor);
  const [marketAddress] = findMarketAddress(marketId);
  const [ledgerAddress] = findBettorLedgerAddress(marketAddress, bettorAddress);
  const account = await connection.getAccountInfo(ledgerAddress);

  if (!account) {
    return emptyBettorResponse(ledgerAddress, bettorAddress, marketAddress);
  }

  return decodeBettor(ledgerAddress, account.data);
}

function decodeConfig(address: PublicKey, data: Buffer): ConfigResponse {
  assertDiscriminator(data, CONFIG_DISCRIMINATOR, "Config");

  let offset = 8;
  const admin = readPublicKey(data, offset);
  offset += 32;
  const treasuryBps = data.readUInt16LE(offset);
  offset += 2;
  const accruedFees = readU64(data, offset);
  offset += 8;
  const nextMarketId = Number(readU64(data, offset));

  return {
    address: address.toBase58(),
    admin: admin.toBase58(),
    treasury_bps: treasuryBps,
    accrued_fees: accruedFees.toString(),
    next_market_id: nextMarketId,
  };
}

function decodeMarket(address: PublicKey, data: Buffer): MarketResponse {
  assertDiscriminator(data, MARKET_DISCRIMINATOR, "Market");

  let offset = 8;
  const marketId = Number(readU64(data, offset));
  offset += 8;
  const league = readAnchorString(data, offset);
  offset += 4 + Buffer.byteLength(league);
  const homeTeam = readAnchorString(data, offset);
  offset += 4 + Buffer.byteLength(homeTeam);
  const awayTeam = readAnchorString(data, offset);
  offset += 4 + Buffer.byteLength(awayTeam);
  const kickoffTs = readI64(data, offset);
  offset += 8;
  const closeTs = readI64(data, offset);
  offset += 8;
  const oracle = readPublicKey(data, offset);
  offset += 32;
  const status = parseStatus(data.readUInt8(offset));
  offset += 1;
  const settledOutcome = readOptionOutcome(data, offset);
  offset += settledOutcome.bytesRead;
  const settledAt = readOptionI64(data, offset);
  offset += settledAt.bytesRead;
  const totalStaked = readU64(data, offset);
  offset += 8;
  const totalPayoutPool = readU64(data, offset);
  offset += 8;
  const totalFee = readU64(data, offset);
  offset += 8;
  const paidOut = readU64(data, offset);
  offset += 8;
  const winningClaimedStake = readU64(data, offset);
  offset += 8;
  const pools = [readU64(data, offset), readU64(data, offset + 8), readU64(data, offset + 16)];

  return {
    address: address.toBase58(),
    market_id: marketId,
    league,
    home_team: homeTeam,
    away_team: awayTeam,
    kickoff_ts: kickoffTs.toString(),
    close_ts: closeTs.toString(),
    oracle: oracle.toBase58(),
    status,
    settled_outcome: settledOutcome.value,
    settled_at: settledAt.value?.toString() ?? null,
    total_staked: totalStaked.toString(),
    total_payout_pool: totalPayoutPool.toString(),
    total_fee: totalFee.toString(),
    paid_out: paidOut.toString(),
    winning_claimed_stake: winningClaimedStake.toString(),
    home_pool: pools[0].toString(),
    draw_pool: pools[1].toString(),
    away_pool: pools[2].toString(),
  };
}

function decodeBettor(address: PublicKey, data: Buffer): BettorResponse {
  assertDiscriminator(data, BETTOR_DISCRIMINATOR, "BettorLedger");

  let offset = 8;
  const bettor = readPublicKey(data, offset);
  offset += 32;
  const market = readPublicKey(data, offset);
  offset += 32;
  const homeStake = readU64(data, offset);
  offset += 8;
  const drawStake = readU64(data, offset);
  offset += 8;
  const awayStake = readU64(data, offset);
  offset += 8;
  const claimed = data.readUInt8(offset) === 1;
  offset += 1;
  const refunded = data.readUInt8(offset) === 1;

  return {
    address: address.toBase58(),
    bettor: bettor.toBase58(),
    market: market.toBase58(),
    home_stake: homeStake.toString(),
    draw_stake: drawStake.toString(),
    away_stake: awayStake.toString(),
    claimed,
    refunded,
  };
}

function emptyBettorResponse(
  address: PublicKey,
  bettor: PublicKey,
  market: PublicKey,
): BettorResponse {
  return {
    address: address.toBase58(),
    bettor: bettor.toBase58(),
    market: market.toBase58(),
    home_stake: "0",
    draw_stake: "0",
    away_stake: "0",
    claimed: false,
    refunded: false,
  };
}

function parseStatus(value: number): MarketStatus {
  switch (value) {
    case 0:
      return "open";
    case 1:
      return "settled";
    case 2:
      return "cancelled";
    default:
      throw new Error(`Unknown market status: ${value}`);
  }
}

function parseOutcome(value: number): Outcome {
  switch (value) {
    case 0:
      return "home_win";
    case 1:
      return "draw";
    case 2:
      return "away_win";
    default:
      throw new Error(`Unknown outcome: ${value}`);
  }
}

function readAnchorString(data: Buffer, offset: number) {
  const length = data.readUInt32LE(offset);
  return data.toString("utf8", offset + 4, offset + 4 + length);
}

function readOptionOutcome(data: Buffer, offset: number) {
  const isSome = data.readUInt8(offset);
  if (isSome === 0) {
    return { value: null, bytesRead: 1 };
  }
  return { value: parseOutcome(data.readUInt8(offset + 1)), bytesRead: 2 };
}

function readOptionI64(data: Buffer, offset: number) {
  const isSome = data.readUInt8(offset);
  if (isSome === 0) {
    return { value: null, bytesRead: 1 };
  }
  return { value: readI64(data, offset + 1), bytesRead: 9 };
}

function readPublicKey(data: Buffer, offset: number) {
  return new PublicKey(data.subarray(offset, offset + 32));
}

function readU64(data: Buffer, offset: number) {
  return data.readBigUInt64LE(offset);
}

function readI64(data: Buffer, offset: number) {
  return data.readBigInt64LE(offset);
}

function assertDiscriminator(data: Buffer, expected: Buffer, name: string) {
  const actual = data.subarray(0, 8);
  if (!actual.equals(expected)) {
    throw new Error(`${name} account discriminator mismatch.`);
  }
}
