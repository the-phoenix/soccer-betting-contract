export function shortenAddress(value: string) {
  if (value.length < 14) {
    return value;
  }
  return `${value.slice(0, 8)}...${value.slice(-6)}`;
}

export function formatTimestamp(value: string | null) {
  if (!value) {
    return "Not set";
  }

  const millis = Number(value) * 1000;
  if (Number.isNaN(millis)) {
    return value;
  }

  return new Date(millis).toLocaleString();
}

export function parseInteger(value: string, field: string) {
  const parsed = Number.parseInt(value, 10);
  if (Number.isNaN(parsed)) {
    throw new Error(`Invalid ${field}.`);
  }
  return parsed;
}

export function parsePublicKey(value: string, field: string) {
  const trimmed = value.trim();
  if (!trimmed) {
    throw new Error(`Missing ${field}.`);
  }
  return trimmed;
}

export function lamportsToSol(value: string) {
  const lamports = Number(value);
  if (Number.isNaN(lamports)) {
    return value;
  }
  return (lamports / 1_000_000_000).toFixed(4);
}
