import { PublicKey } from "@solana/web3.js";

type SolanaWallet = {
  isPhantom?: boolean;
  isBackpack?: boolean;
  publicKey?: PublicKey;
  connect: () => Promise<{ publicKey: PublicKey }>;
  disconnect?: () => Promise<void>;
  signTransaction?: <T>(transaction: T) => Promise<T>;
  signAllTransactions?: <T>(transactions: T[]) => Promise<T[]>;
};

type SolanaWindow = Window & {
  solana?: SolanaWallet;
  backpack?: {
    solana?: SolanaWallet;
  };
};

export function getInjectedWallet() {
  const browserWindow = window as SolanaWindow;
  const candidates = [
    browserWindow.solana,
    browserWindow.backpack?.solana,
  ].filter((wallet): wallet is SolanaWallet => Boolean(wallet));

  const wallet = candidates[0];
  if (!wallet) {
    throw new Error("No injected Solana wallet was found in this browser.");
  }

  return wallet;
}

export async function connectWallet() {
  const wallet = getInjectedWallet();
  const result = await wallet.connect();
  const publicKey = result.publicKey ?? wallet.publicKey;

  if (!publicKey) {
    throw new Error("The connected wallet did not expose a public key.");
  }

  return {
    wallet,
    publicKey,
    address: publicKey.toBase58(),
  };
}

export async function disconnectWallet() {
  const wallet = getInjectedWallet();
  if (wallet.disconnect) {
    await wallet.disconnect();
  }
}

export function hasSigningSupport(wallet: SolanaWallet) {
  return Boolean(wallet.signTransaction);
}

export type { SolanaWallet };
