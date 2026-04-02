import type { OfflineSigner } from "@cosmjs/proto-signing";

import { appConfig } from "./config";

type KeplrWindow = Window & {
  keplr?: {
    enable: (chainId: string) => Promise<void>;
    getOfflineSignerAuto: (chainId: string) => Promise<OfflineSigner>;
  };
  getOfflineSigner?: (chainId: string) => OfflineSigner;
};

export async function connectKeplr() {
  const browserWindow = window as KeplrWindow;
  if (!browserWindow.keplr) {
    throw new Error("Keplr wallet was not found in this browser.");
  }
  if (!appConfig.chainId) {
    throw new Error("NEXT_PUBLIC_CHAIN_ID is not configured.");
  }

  await browserWindow.keplr.enable(appConfig.chainId);
  const signer = await browserWindow.keplr.getOfflineSignerAuto(appConfig.chainId);
  const accounts = await signer.getAccounts();
  const primaryAccount = accounts[0];

  if (!primaryAccount) {
    throw new Error("No wallet account is available.");
  }

  return {
    signer,
    address: primaryAccount.address,
  };
}
