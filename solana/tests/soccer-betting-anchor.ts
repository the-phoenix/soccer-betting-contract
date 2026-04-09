import { strict as assert } from "node:assert";
import { setTimeout as delay } from "node:timers/promises";

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";

describe("soccer-betting-anchor", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .SoccerBettingAnchor as Program;
  const programAny = program as any;

  const configSeeds = [Buffer.from("config")];

  async function airdrop(publicKey: PublicKey, amount = LAMPORTS_PER_SOL) {
    const signature = await provider.connection.requestAirdrop(publicKey, amount);
    const latestBlockhash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction(
      {
        signature,
        ...latestBlockhash,
      },
      "confirmed",
    );
  }

  it("creates and settles a market", async () => {
    const walletPublicKey = provider.wallet.publicKey;
    const oracle = Keypair.generate();
    const alice = Keypair.generate();
    const bob = Keypair.generate();

    await Promise.all([
      airdrop(oracle.publicKey),
      airdrop(alice.publicKey),
      airdrop(bob.publicKey),
    ]);

    const [configPda] = PublicKey.findProgramAddressSync(
      configSeeds,
      program.programId,
    );

    await program.methods
      .initialize(null, 250)
      .accountsPartial({
        config: configPda,
        payer: walletPublicKey,
      })
      .rpc();

    const configAccount = await programAny.account.config.fetch(configPda);
    assert.equal(configAccount.treasuryBps, 250);
    assert.equal(configAccount.nextMarketId.toNumber(), 1);

    const marketId = configAccount.nextMarketId.toNumber();
    const marketSeed = new anchor.BN(marketId).toArrayLike(Buffer, "le", 8);
    const [marketPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("market"), marketSeed],
      program.programId,
    );

    const now = Math.floor(Date.now() / 1000);
    const kickoffTs = new anchor.BN(now + 2);
    const closeTs = new anchor.BN(now + 1);

    await program.methods
      .createMarket(
        "Premier League",
        "Arsenal",
        "Liverpool",
        kickoffTs,
        closeTs,
        oracle.publicKey,
      )
      .accountsPartial({
        config: configPda,
        market: marketPda,
        admin: walletPublicKey,
      })
      .rpc();

    const [aliceLedgerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bettor"), marketPda.toBuffer(), alice.publicKey.toBuffer()],
      program.programId,
    );
    const [bobLedgerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bettor"), marketPda.toBuffer(), bob.publicKey.toBuffer()],
      program.programId,
    );

    await program.methods
      .placeBet(new anchor.BN(marketId), 0, new anchor.BN(100_000_000))
      .accountsPartial({
        config: configPda,
        market: marketPda,
        bettorLedger: aliceLedgerPda,
        bettor: alice.publicKey,
      })
      .signers([alice])
      .rpc();

    await program.methods
      .placeBet(new anchor.BN(marketId), 1, new anchor.BN(50_000_000))
      .accountsPartial({
        config: configPda,
        market: marketPda,
        bettorLedger: bobLedgerPda,
        bettor: bob.publicKey,
      })
      .signers([bob])
      .rpc();

    const earlySettle = await program.methods
      .settleMarket(new anchor.BN(marketId), 0)
      .accountsPartial({
        config: configPda,
        market: marketPda,
        authority: oracle.publicKey,
      })
      .signers([oracle])
      .rpc()
      .then(() => null)
      .catch((error) => error);

    assert(earlySettle instanceof Error);

    const marketBeforeWarp = await programAny.account.market.fetch(marketPda);
    assert.equal(marketBeforeWarp.totalStaked.toNumber(), 150_000_000);
    await delay(3_000);

    await program.methods
      .settleMarket(new anchor.BN(marketId), 0)
      .accountsPartial({
        config: configPda,
        market: marketPda,
        authority: oracle.publicKey,
      })
      .signers([oracle])
      .rpc();

    const marketAccount = await programAny.account.market.fetch(marketPda);
    assert.equal(marketAccount.status, 1);
    assert.equal(marketAccount.totalStaked.toNumber(), 150_000_000);
    assert.equal(marketAccount.totalFee.toNumber(), 3_750_000);
    assert.equal(marketAccount.totalPayoutPool.toNumber(), 146_250_000);
  });
});
