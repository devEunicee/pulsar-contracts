import React, { useState } from "react";
import { useContractCall } from "../hooks/useContractCall";
import { Spinner, ErrorMessage, SuccessBanner } from "./ContractCallStatus";

export interface CheckoutWidgetProps {
  orderId: string;
  amount: number;
  token: string;
  merchantAddress: string;
  onSuccess?: (txHash: string) => void;
  onError?: (message: string) => void;
}

/**
 * Embeddable checkout widget.
 * Handles wallet connection, signing, and contract submission.
 * Emits onSuccess / onError callbacks.
 */
export function CheckoutWidget({
  orderId,
  amount,
  token,
  merchantAddress,
  onSuccess,
  onError,
}: CheckoutWidgetProps) {
  const [walletAddress, setWalletAddress] = useState<string | null>(null);
  const { state, execute, reset } = useContractCall<null>();

  async function connectWallet() {
    // Freighter wallet integration
    const freighter = (window as any).freighter;
    if (!freighter) {
      alert("Freighter wallet extension not found. Please install it.");
      return;
    }
    await freighter.setAllowed();
    const address = await freighter.getPublicKey();
    setWalletAddress(address);
  }

  async function handlePay() {
    if (!walletAddress) return;
    await execute(async () => {
      const freighter = (window as any).freighter;

      // Build the order payload
      const order = {
        order_id: orderId,
        merchant_address: merchantAddress,
        payer: walletAddress,
        token,
        amount,
        description: `Payment for order ${orderId}`,
        expires_at: Math.floor(Date.now() / 1000) + 3600,
      };

      // Sign the order with the merchant key via Freighter
      const orderJson = JSON.stringify(order);
      const { signedXDR } = await freighter.signTransaction(orderJson, {
        network: "TESTNET",
      });

      // Submit to contract (replace with your SDK call)
      const txHash = await submitPayment({ order, signature: signedXDR });

      onSuccess?.(txHash);
      return { result: null, txHash };
    });

    if (state.status === "error") {
      onError?.((state as any).message);
    }
  }

  return (
    <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 max-w-sm w-full shadow-sm space-y-4">
      <h2 className="text-lg font-semibold text-[var(--color-text)]">Checkout</h2>

      <dl className="text-sm space-y-1 text-[var(--color-text-muted)]">
        <div className="flex justify-between">
          <dt>Order</dt>
          <dd className="font-mono text-[var(--color-text)]">{orderId}</dd>
        </div>
        <div className="flex justify-between">
          <dt>Amount</dt>
          <dd className="font-medium text-[var(--color-text)]">{amount}</dd>
        </div>
        <div className="flex justify-between">
          <dt>Token</dt>
          <dd className="font-mono truncate max-w-[160px] text-[var(--color-text)]">{token}</dd>
        </div>
      </dl>

      {state.status === "success" && <SuccessBanner txHash={state.txHash} />}
      {state.status === "error" && <ErrorMessage message={state.message} />}

      {state.status !== "success" && (
        <>
          {!walletAddress ? (
            <button
              onClick={connectWallet}
              className="w-full rounded-lg bg-[var(--color-primary)] hover:bg-[var(--color-primary-hover)] text-white py-2 px-4 text-sm font-medium transition-colors"
            >
              Connect Wallet
            </button>
          ) : (
            <div className="space-y-2">
              <p className="text-xs text-[var(--color-text-muted)] truncate">
                Connected: <span className="font-mono">{walletAddress}</span>
              </p>
              <button
                onClick={handlePay}
                disabled={state.status === "loading"}
                className="w-full rounded-lg bg-[var(--color-primary)] hover:bg-[var(--color-primary-hover)] disabled:opacity-50 text-white py-2 px-4 text-sm font-medium transition-colors flex items-center justify-center gap-2"
              >
                {state.status === "loading" ? <Spinner label="Submitting…" /> : "Pay Now"}
              </button>
            </div>
          )}
        </>
      )}

      {state.status === "error" && (
        <button onClick={reset} className="text-xs text-[var(--color-text-muted)] underline">
          Try again
        </button>
      )}
    </div>
  );
}

/** Stub — replace with actual @stellar/stellar-sdk contract invocation. */
async function submitPayment(_payload: { order: object; signature: string }): Promise<string> {
  throw new Error("submitPayment: not yet wired to contract SDK.");
}
