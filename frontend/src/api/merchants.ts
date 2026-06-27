import { apiClient } from "./client";
import type { Merchant } from "../store/merchantsStore";

export const merchantsApi = {
  register: (data: Omit<Merchant, "active">) =>
    apiClient.post<Merchant>("/merchants", data),

  get: (address: string) =>
    apiClient.get<Merchant>(`/merchants/${address}`),

  deactivate: (address: string) =>
    apiClient.patch<void>(`/merchants/${address}/deactivate`, {}),
};
