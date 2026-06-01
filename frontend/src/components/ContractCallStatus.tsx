import React from "react";

export function Spinner({ label = "Loading…" }: { label?: string }) {
  return (
    <span role="status" aria-label={label} className="inline-flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
      <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
      </svg>
      {label}
    </span>
  );
}

export function ErrorMessage({ message }: { message: string }) {
  return (
    <div role="alert" className="rounded-md bg-red-50 dark:bg-red-900/30 border border-red-300 dark:border-red-700 p-3 text-sm text-red-700 dark:text-red-300">
      {message}
    </div>
  );
}

const EXPLORER_BASE = "https://stellar.expert/explorer/testnet/tx";

export function SuccessBanner({ txHash }: { txHash: string }) {
  return (
    <div role="status" className="rounded-md bg-green-50 dark:bg-green-900/30 border border-green-300 dark:border-green-700 p-3 text-sm text-green-700 dark:text-green-300">
      Transaction submitted!{" "}
      <a
        href={`${EXPLORER_BASE}/${txHash}`}
        target="_blank"
        rel="noopener noreferrer"
        className="underline font-medium"
      >
        View on Stellar Explorer ↗
      </a>
    </div>
  );
}
