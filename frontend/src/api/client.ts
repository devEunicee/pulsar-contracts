const BASE_URL = process.env.REACT_APP_API_URL ?? "http://localhost:3000";
const TIMEOUT_MS = 10_000;
const MAX_RETRIES = 3;
const RETRYABLE_STATUSES = new Set([429, 502, 503, 504]);

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly body: unknown,
    message: string
  ) {
    super(message);
    this.name = "ApiError";
  }
}

function getToken(): string | null {
  return localStorage.getItem("auth_token");
}

function buildHeaders(extra?: HeadersInit): Headers {
  const headers = new Headers({ "Content-Type": "application/json", ...(extra as Record<string, string>) });
  const token = getToken();
  if (token) headers.set("Authorization", `Bearer ${token}`);
  return headers;
}

async function parseResponse<T>(res: Response): Promise<T> {
  const text = await res.text();
  let body: unknown;
  try { body = JSON.parse(text); } catch { body = text; }
  if (!res.ok) throw new ApiError(res.status, body, `HTTP ${res.status}`);
  return body as T;
}

async function fetchWithTimeout(
  url: string,
  init: RequestInit
): Promise<Response> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), TIMEOUT_MS);
  try {
    return await fetch(url, { ...init, signal: controller.signal });
  } finally {
    clearTimeout(timer);
  }
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
  attempt = 1
): Promise<T> {
  const url = `${BASE_URL}${path}`;
  const init: RequestInit = {
    method,
    headers: buildHeaders(),
    ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
  };

  if (process.env.NODE_ENV === "development") {
    console.debug(`[API] ${method} ${url}`, body ?? "");
  }

  let res: Response;
  try {
    res = await fetchWithTimeout(url, init);
  } catch (err) {
    if (attempt < MAX_RETRIES) {
      await delay(200 * attempt);
      return request<T>(method, path, body, attempt + 1);
    }
    throw err;
  }

  if (RETRYABLE_STATUSES.has(res.status) && attempt < MAX_RETRIES) {
    await delay(200 * attempt);
    return request<T>(method, path, body, attempt + 1);
  }

  const data = await parseResponse<T>(res);

  if (process.env.NODE_ENV === "development") {
    console.debug(`[API] ${res.status} ${url}`, data);
  }

  return data;
}

const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

export const apiClient = {
  get: <T>(path: string) => request<T>("GET", path),
  post: <T>(path: string, body: unknown) => request<T>("POST", path, body),
  put: <T>(path: string, body: unknown) => request<T>("PUT", path, body),
  patch: <T>(path: string, body: unknown) => request<T>("PATCH", path, body),
  delete: <T>(path: string) => request<T>("DELETE", path),
};
