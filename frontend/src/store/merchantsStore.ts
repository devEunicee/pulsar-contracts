import { create } from "zustand";
import { devtools } from "zustand/middleware";

export interface Merchant {
  address: string;
  name: string;
  description: string;
  contactInfo: string;
  category: string;
  active: boolean;
}

interface MerchantsState {
  merchants: Merchant[];
  selected: Merchant | null;
  isLoading: boolean;
  error: string | null;
  setMerchants: (merchants: Merchant[]) => void;
  upsertMerchant: (merchant: Merchant) => void;
  selectMerchant: (merchant: Merchant | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

export const useMerchantsStore = create<MerchantsState>()(
  devtools(
    (set) => ({
      merchants: [],
      selected: null,
      isLoading: false,
      error: null,
      setMerchants: (merchants) => set({ merchants }),
      upsertMerchant: (merchant) =>
        set((s) => ({
          merchants: s.merchants.some((m) => m.address === merchant.address)
            ? s.merchants.map((m) => (m.address === merchant.address ? merchant : m))
            : [...s.merchants, merchant],
        })),
      selectMerchant: (selected) => set({ selected }),
      setLoading: (isLoading) => set({ isLoading }),
      setError: (error) => set({ error, isLoading: false }),
    }),
    { name: "merchants" }
  )
);

// Selectors
export const selectMerchants = (s: MerchantsState) => s.merchants;
export const selectActiveMerchants = (s: MerchantsState) =>
  s.merchants.filter((m) => m.active);
export const selectSelectedMerchant = (s: MerchantsState) => s.selected;
export const selectMerchantsLoading = (s: MerchantsState) => s.isLoading;
