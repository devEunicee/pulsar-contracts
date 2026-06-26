import { apiClient } from "./client";
import type { Payment } from "../store/paymentsStore";

export interface PaymentHistoryParams {
  cursor?: string;
  limit?: number;
  sortField?: "Date" | "Amount";
  sortOrder?: "Ascending" | "Descending";
}

export interface PaymentHistoryResponse {
  payments: Payment[];
  nextCursor: string | null;
}

export const paymentsApi = {
  getById: (orderId: string) =>
    apiClient.get<Payment>(`/payments/${orderId}`),

  getMerchantHistory: (address: string, params: PaymentHistoryParams = {}) => {
    const qs = new URLSearchParams(params as Record<string, string>).toString();
    return apiClient.get<PaymentHistoryResponse>(`/payments/merchant/${address}?${qs}`);
  },

  getPayerHistory: (payer: string, params: PaymentHistoryParams = {}) => {
    const qs = new URLSearchParams(params as Record<string, string>).toString();
    return apiClient.get<PaymentHistoryResponse>(`/payments/payer/${payer}?${qs}`);
  },
};
