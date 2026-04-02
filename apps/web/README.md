# PitchPool Web

Next.js frontend for the soccer betting CosmWasm contract.

## Features

- Keplr wallet connection
- Contract config query
- Market query by id
- Bettor query by market id and address
- Execute flows for:
  - `create_market`
  - `place_bet`
  - `settle_market`
  - `cancel_market`
  - `claim`
  - `refund`
  - `withdraw_fees`

## Setup

```bash
cp .env.example .env.local
npm install
npm run dev
```

Required `env` values:

- `NEXT_PUBLIC_CHAIN_NAME`
- `NEXT_PUBLIC_CHAIN_ID`
- `NEXT_PUBLIC_RPC_ENDPOINT`
- `NEXT_PUBLIC_CONTRACT_ADDRESS`
- `NEXT_PUBLIC_STAKE_DENOM`

Optional:

- `NEXT_PUBLIC_REST_ENDPOINT`
- `NEXT_PUBLIC_ADDRESS_PREFIX`

## Notes

- The UI expects a Keplr-compatible wallet in the browser.
- Execute actions use the configured `NEXT_PUBLIC_CONTRACT_ADDRESS`.
- `place_bet` sends native funds in `NEXT_PUBLIC_STAKE_DENOM`.
