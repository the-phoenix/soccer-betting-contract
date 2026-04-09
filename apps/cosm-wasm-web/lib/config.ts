export const appConfig = {
  chainName: process.env.NEXT_PUBLIC_CHAIN_NAME ?? "Local Cosmos",
  chainId: process.env.NEXT_PUBLIC_CHAIN_ID ?? "",
  rpcEndpoint: process.env.NEXT_PUBLIC_RPC_ENDPOINT ?? "",
  restEndpoint: process.env.NEXT_PUBLIC_REST_ENDPOINT ?? "",
  contractAddress: process.env.NEXT_PUBLIC_CONTRACT_ADDRESS ?? "",
  stakeDenom: process.env.NEXT_PUBLIC_STAKE_DENOM ?? "ucosm",
  addressPrefix: process.env.NEXT_PUBLIC_ADDRESS_PREFIX ?? "cosmos",
};

export function hasRequiredChainConfig() {
  return Boolean(
    appConfig.chainId && appConfig.rpcEndpoint && appConfig.contractAddress,
  );
}
