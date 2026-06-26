import { create } from "zustand";
import { devtools } from "zustand/middleware";

export interface Payment {
  orderId: string;
  merchantAddress: string;
  payer: string;
  token: string;
  amount: number;
  status: "Completed" | "PartiallyRefunded" | "FullyRefunded";
  paidAt: number;
}

interface PaymentsState {
  payments: Payment[];
  nextCursor: string | null;
  isLoading: boolean;
  error: string | null;
  setPayments: (payments: Payment[], nextCursor: string | null) => void;
  appendPayments: (payments: Payment[], nextCursor: string | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

export const usePaymentsStore = create<PaymentsState>()(
  devtools(
    (set) => ({
      payments: [],
      nextCursor: null,
      isLoading: false,
      error: null,
      setPayments: (payments, nextCursor) => set({ payments, nextCursor }),
      appendPayments: (payments, nextCursor) =>
        set((s) => ({ payments: [...s.payments, ...payments], nextCursor })),
      setLoading: (isLoading) => set({ isLoading }),
      setError: (error) => set({ error, isLoading: false }),
    }),
    { name: "payments" }
  )
);

// Selectors
export const selectPayments = (s: PaymentsState) => s.payments;
export const selectNextCursor = (s: PaymentsState) => s.nextCursor;
export const selectPaymentsLoading = (s: PaymentsState) => s.isLoading;
export const selectPaymentsError = (s: PaymentsState) => s.error;
