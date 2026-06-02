import { Contract, Networks, rpc } from "@stellar/stellar-sdk";

export const CONTRACT_ID = import.meta.env.VITE_CONTRACT_ID as string;
export const NETWORK_PASSPHRASE = import.meta.env.VITE_NETWORK_PASSPHRASE ?? Networks.TESTNET;
export const RPC_URL = import.meta.env.VITE_RPC_URL ?? "https://soroban-testnet.stellar.org";

export const server = new rpc.Server(RPC_URL);
export const contract = new Contract(CONTRACT_ID);

export interface PaymentRecord {
  order_id: string;
  merchant_address: string;
  payer: string;
  token: string;
  amount: string;
  paid_at: number;
  status: string;
}

export interface PaymentPage {
  records: PaymentRecord[];
  next_cursor: string | null;
}

export interface PaymentFilter {
  date_start?: number;
  date_end?: number;
  amount_min?: number;
  amount_max?: number;
  status?: "Any" | "Completed" | "PartiallyRefunded" | "FullyRefunded";
}

export async function fetchPayerHistory(
  payer: string,
  cursor: string | null,
  limit: number,
  filter: PaymentFilter,
  sort_field: "Date" | "Amount",
  sort_order: "Ascending" | "Descending"
): Promise<PaymentPage> {
  const tx = contract.call(
    "get_payer_payment_history",
    ...[payer, cursor, limit, filter, sort_field, sort_order].map((v) =>
      JSON.stringify(v)
    )
  );
  const result = await server.simulateTransaction(
    await server.prepareTransaction(tx as any)
  );
  // @ts-expect-error result shape varies by SDK version
  return result.result?.retval?.value() ?? { records: [], next_cursor: null };
}
