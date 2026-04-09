# Soccer Betting Platform

CosmWasm contract and Next.js frontend for pooled 1X2 soccer betting markets using a single native token denom.

## Layout

- `contracts/soccer-betting-contract`
  CosmWasm smart contract for pooled 1X2 soccer betting, settlement, cancellation, claims, and refunds.
- `apps/cosm-wasm-web`
  Next.js frontend for market exploration, wallet actions, queries, and operator flows.

## Current Status

The contract is implemented and tested. The frontend is implemented and wired to the contract message/query surface.

## Contract Quick Start

```bash
cd contracts/soccer-betting-contract
cargo test
cargo run --bin schema
```

## Frontend Quick Start

```bash
cd apps/cosm-wasm-web
cp .env.example .env.local
npm install
npm run dev
```
