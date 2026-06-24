import { useState, useCallback } from "react";
import {
  isConnected,
  getAddress,
  requestAccess,
} from "@stellar/freighter-api";

export function useWallet() {
  const [address, setAddress] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const connect = useCallback(async () => {
    setError(null);
    try {
      const connected = await isConnected();
      if (!connected) {
        setError("Freighter extension not found. Please install it.");
        return;
      }
      await requestAccess();
      const { address: addr } = await getAddress();
      setAddress(addr);
    } catch (e: any) {
      setError(e?.message ?? "Failed to connect wallet");
    }
  }, []);

  const disconnect = useCallback(() => setAddress(null), []);

  return { address, error, connect, disconnect };
}
