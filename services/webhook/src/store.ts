export interface WebhookRegistration {
  merchantAddress: string;
  url: string;
  registeredAt: number;
}

// In-memory store — replace with a database in production.
const store = new Map<string, WebhookRegistration>();

export function registerWebhook(merchantAddress: string, url: string): WebhookRegistration {
  const reg: WebhookRegistration = { merchantAddress, url, registeredAt: Date.now() };
  store.set(merchantAddress, reg);
  return reg;
}

export function getWebhook(merchantAddress: string): WebhookRegistration | undefined {
  return store.get(merchantAddress);
}

export function deleteWebhook(merchantAddress: string): boolean {
  return store.delete(merchantAddress);
}

export function allWebhooks(): WebhookRegistration[] {
  return Array.from(store.values());
}
