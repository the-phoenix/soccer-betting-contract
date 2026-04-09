import { Buffer } from "buffer";
import {
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";

import {
  findBettorLedgerAddress,
  findConfigAddress,
  findMarketAddress,
  getConnection,
  getProgramId,
  queryConfig,
} from "./anchor-client";
import type { Outcome } from "./contract-types";
import type { SolanaWallet } from "./wallet";

const INITIALIZE_IX = Buffer.from("afaf6d1f0d989bed", "hex");
const CREATE_MARKET_IX = Buffer.from("67e261ebc8bcfbfe", "hex");
const PLACE_BET_IX = Buffer.from("de3e43dc3fa67e21", "hex");
const SETTLE_MARKET_IX = Buffer.from("c1995fd8a60690d9", "hex");
const CANCEL_MARKET_IX = Buffer.from("cd7954d2de47960b", "hex");
const CLAIM_IX = Buffer.from("3ec6d6c1d59f6cd2", "hex");
const REFUND_IX = Buffer.from("0260b7fb3fd02e2e", "hex");
const WITHDRAW_FEES_IX = Buffer.from("c6d4ab6d90d7ae59", "hex");

type SignAndSendArgs = {
  wallet: SolanaWallet;
  signerAddress: string;
  instruction: TransactionInstruction;
};

type CreateMarketArgs = {
  signerAddress: string;
  league: string;
  homeTeam: string;
  awayTeam: string;
  kickoffTs: number;
  closeTs: number;
  oracle: string;
};

export async function initializeProgram(
  signerAddress: string,
  wallet: SolanaWallet,
  treasuryBps: number,
  admin?: string,
) {
  const [configAddress] = findConfigAddress();

  return signAndSend({
    wallet,
    signerAddress,
    instruction: new TransactionInstruction({
      programId: getProgramId(),
      keys: [
        { pubkey: configAddress, isSigner: false, isWritable: true },
        { pubkey: new PublicKey(signerAddress), isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data: Buffer.concat([
        INITIALIZE_IX,
        encodeOptionPublicKey(admin),
        encodeU16(treasuryBps),
      ]),
    }),
  });
}

export async function createMarket(args: CreateMarketArgs, wallet: SolanaWallet) {
  const [configAddress] = findConfigAddress();
  const config = await queryConfig();
  const [marketAddress] = findMarketAddress(config.next_market_id);

  return signAndSend({
    wallet,
    signerAddress: args.signerAddress,
    instruction: new TransactionInstruction({
      programId: getProgramId(),
      keys: [
        { pubkey: configAddress, isSigner: false, isWritable: true },
        { pubkey: marketAddress, isSigner: false, isWritable: true },
        { pubkey: new PublicKey(args.signerAddress), isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data: Buffer.concat([
        CREATE_MARKET_IX,
        encodeString(args.league),
        encodeString(args.homeTeam),
        encodeString(args.awayTeam),
        encodeI64(args.kickoffTs),
        encodeI64(args.closeTs),
        new PublicKey(args.oracle).toBuffer(),
      ]),
    }),
  });
}

export async function placeBet(
  signerAddress: string,
  wallet: SolanaWallet,
  marketId: number,
  outcome: Outcome,
  stakeLamports: number,
) {
  const [configAddress] = findConfigAddress();
  const [marketAddress] = findMarketAddress(marketId);
  const bettorAddress = new PublicKey(signerAddress);
  const [bettorLedgerAddress] = findBettorLedgerAddress(marketAddress, bettorAddress);

  return signAndSend({
    wallet,
    signerAddress,
    instruction: new TransactionInstruction({
      programId: getProgramId(),
      keys: [
        { pubkey: configAddress, isSigner: false, isWritable: false },
        { pubkey: marketAddress, isSigner: false, isWritable: true },
        { pubkey: bettorLedgerAddress, isSigner: false, isWritable: true },
        { pubkey: bettorAddress, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data: Buffer.concat([
        PLACE_BET_IX,
        encodeU64(marketId),
        encodeU8(encodeOutcome(outcome)),
        encodeU64(stakeLamports),
      ]),
    }),
  });
}

export async function settleMarket(
  signerAddress: string,
  wallet: SolanaWallet,
  marketId: number,
  outcome: Outcome,
) {
  const [configAddress] = findConfigAddress();
  const [marketAddress] = findMarketAddress(marketId);

  return signAndSend({
    wallet,
    signerAddress,
    instruction: new TransactionInstruction({
      programId: getProgramId(),
      keys: [
        { pubkey: configAddress, isSigner: false, isWritable: true },
        { pubkey: marketAddress, isSigner: false, isWritable: true },
        { pubkey: new PublicKey(signerAddress), isSigner: true, isWritable: false },
      ],
      data: Buffer.concat([
        SETTLE_MARKET_IX,
        encodeU64(marketId),
        encodeU8(encodeOutcome(outcome)),
      ]),
    }),
  });
}

export async function cancelMarket(
  signerAddress: string,
  wallet: SolanaWallet,
  marketId: number,
) {
  const [configAddress] = findConfigAddress();
  const [marketAddress] = findMarketAddress(marketId);

  return signAndSend({
    wallet,
    signerAddress,
    instruction: new TransactionInstruction({
      programId: getProgramId(),
      keys: [
        { pubkey: configAddress, isSigner: false, isWritable: false },
        { pubkey: marketAddress, isSigner: false, isWritable: true },
        { pubkey: new PublicKey(signerAddress), isSigner: true, isWritable: false },
      ],
      data: Buffer.concat([CANCEL_MARKET_IX, encodeU64(marketId)]),
    }),
  });
}

export async function claim(
  signerAddress: string,
  wallet: SolanaWallet,
  marketId: number,
) {
  const [marketAddress] = findMarketAddress(marketId);
  const bettorAddress = new PublicKey(signerAddress);
  const [bettorLedgerAddress] = findBettorLedgerAddress(marketAddress, bettorAddress);

  return signAndSend({
    wallet,
    signerAddress,
    instruction: new TransactionInstruction({
      programId: getProgramId(),
      keys: [
        { pubkey: marketAddress, isSigner: false, isWritable: true },
        { pubkey: bettorLedgerAddress, isSigner: false, isWritable: true },
        { pubkey: bettorAddress, isSigner: true, isWritable: true },
      ],
      data: Buffer.concat([CLAIM_IX, encodeU64(marketId)]),
    }),
  });
}

export async function refund(
  signerAddress: string,
  wallet: SolanaWallet,
  marketId: number,
) {
  const [marketAddress] = findMarketAddress(marketId);
  const bettorAddress = new PublicKey(signerAddress);
  const [bettorLedgerAddress] = findBettorLedgerAddress(marketAddress, bettorAddress);

  return signAndSend({
    wallet,
    signerAddress,
    instruction: new TransactionInstruction({
      programId: getProgramId(),
      keys: [
        { pubkey: marketAddress, isSigner: false, isWritable: true },
        { pubkey: bettorLedgerAddress, isSigner: false, isWritable: true },
        { pubkey: bettorAddress, isSigner: true, isWritable: true },
      ],
      data: Buffer.concat([REFUND_IX, encodeU64(marketId)]),
    }),
  });
}

export async function withdrawFees(signerAddress: string, wallet: SolanaWallet) {
  const [configAddress] = findConfigAddress();

  return signAndSend({
    wallet,
    signerAddress,
    instruction: new TransactionInstruction({
      programId: getProgramId(),
      keys: [
        { pubkey: configAddress, isSigner: false, isWritable: true },
        { pubkey: new PublicKey(signerAddress), isSigner: true, isWritable: true },
      ],
      data: WITHDRAW_FEES_IX,
    }),
  });
}

async function signAndSend({ wallet, signerAddress, instruction }: SignAndSendArgs) {
  if (!wallet.signTransaction) {
    throw new Error("This wallet does not support transaction signing.");
  }

  const connection = getConnection();
  const latestBlockhash = await connection.getLatestBlockhash("confirmed");
  const transaction = new Transaction({
    feePayer: new PublicKey(signerAddress),
    blockhash: latestBlockhash.blockhash,
    lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
  }).add(instruction);

  const signed = await wallet.signTransaction(transaction);
  const signature = await connection.sendRawTransaction(signed.serialize());
  await connection.confirmTransaction(
    {
      signature,
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    },
    "confirmed",
  );

  return { signature };
}

function encodeOutcome(outcome: Outcome) {
  switch (outcome) {
    case "home_win":
      return 0;
    case "draw":
      return 1;
    case "away_win":
      return 2;
  }
}

function encodeOptionPublicKey(value?: string) {
  if (!value) {
    return Buffer.from([0]);
  }
  return Buffer.concat([Buffer.from([1]), new PublicKey(value).toBuffer()]);
}

function encodeString(value: string) {
  const text = Buffer.from(value, "utf8");
  return Buffer.concat([encodeU32(text.length), text]);
}

function encodeU8(value: number) {
  return Buffer.from([value]);
}

function encodeU16(value: number) {
  const buffer = Buffer.alloc(2);
  buffer.writeUInt16LE(value);
  return buffer;
}

function encodeU32(value: number) {
  const buffer = Buffer.alloc(4);
  buffer.writeUInt32LE(value);
  return buffer;
}

function encodeU64(value: number | bigint) {
  const buffer = Buffer.alloc(8);
  buffer.writeBigUInt64LE(BigInt(value));
  return buffer;
}

function encodeI64(value: number | bigint) {
  const buffer = Buffer.alloc(8);
  buffer.writeBigInt64LE(BigInt(value));
  return buffer;
}
