export type SearchResultType = "Merchant" | "Payment" | "Customer";

export interface SearchResult {
  id: string;
  type: SearchResultType;
  title: string;
  subtitle?: string;
}

export interface GroupedResults {
  Merchants: SearchResult[];
  Payments: SearchResult[];
  Customers: SearchResult[];
}
