export const appConfig = {
  clusterName: process.env.NEXT_PUBLIC_SOLANA_CLUSTER_NAME ?? "Localnet",
  rpcEndpoint: process.env.NEXT_PUBLIC_SOLANA_RPC_ENDPOINT ?? "",
  programId: process.env.NEXT_PUBLIC_PROGRAM_ID ?? "",
  defaultMarketLimit: Number.parseInt(
    process.env.NEXT_PUBLIC_DEFAULT_MARKET_LIMIT ?? "12",
    10,
  ),
};

export function hasRequiredChainConfig() {
  return Boolean(appConfig.rpcEndpoint && appConfig.programId);
}
