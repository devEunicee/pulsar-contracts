import { useState, useCallback } from 'react';
import {
  isConnected,
  getPublicKey,
  signTransaction,
} from '@stellar/freighter-api';

export function useWallet() {
  const [publicKey, setPublicKey] = useState(null);
  const [error, setError] = useState(null);

  const connect = useCallback(async () => {
    try {
      if (!(await isConnected())) {
        setError('Freighter wallet not found. Please install it.');
        return;
      }
      const key = await getPublicKey();
      setPublicKey(key);
      setError(null);
    } catch (e) {
      setError(e.message);
    }
  }, []);

  const disconnect = useCallback(() => setPublicKey(null), []);

  return { publicKey, connect, disconnect, signTransaction, error };
}
