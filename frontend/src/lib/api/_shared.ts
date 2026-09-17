export interface IBackendStatus {
    status: string;
}

/**
 * Discriminator for errors raised by `backendFetch` and `parseError`.
 *
 * Errors thrown inside a server function reach the browser through seroval,
 * which keeps an Error's own enumerable fields (`kind`, `status`, `code`) and
 * DROPS its class: the client receives a plain `Error`. So every consumer
 * discriminates on `kind`, never on `instanceof`. Fields are assigned in the
 * constructor for the same reason: a `declare`d or prototype field would not
 * cross the boundary.
 */
export type BackendErrorKind = "unreachable" | "http";

/** The backend answered with a non-2xx status. `code` is its `error.code` when the body carried one. */
export class APIError extends Error {
    readonly kind: BackendErrorKind = "http";
    readonly code: string | null;
    constructor(
        public readonly status: number,
        message: string,
        code: string | null = null,
    ) {
        super(message);
        this.name = "APIError";
        this.code = code;
    }
}

/**
 * `fetch` itself threw: the backend never answered. `code` is the socket-level
 * reason when the runtime exposes one (`ECONNREFUSED`, `ENOTFOUND`,
 * `UND_ERR_CONNECT_TIMEOUT` from undici; browsers expose none).
 *
 * The original error is deliberately NOT attached as `cause`: seroval ships
 * `cause` to the browser too, and undici's carries the backend's internal
 * address and port. `backendFetch` logs it server-side instead.
 */
export class BackendUnreachableError extends Error {
    readonly kind: BackendErrorKind = "unreachable";
    readonly code: string | null;
    constructor(cause: unknown) {
        const code = networkErrorCode(cause);
        super(code ? `The server could not be reached (${code}).` : "The server could not be reached.");
        this.name = "BackendUnreachableError";
        this.code = code;
    }
}

export function networkErrorCode(err: unknown): string | null {
    if (!err || typeof err !== "object") return null;
    const e = err as { code?: unknown; name?: unknown; cause?: unknown };
    if (typeof e.code === "string" && e.code) return e.code;
    if (e.cause && e.cause !== err) return networkErrorCode(e.cause);
    if (e.name === "AbortError" || e.name === "TimeoutError") return "TIMEOUT";
    return null;
}

export async function parseError(res: Response): Promise<APIError> {
    const fallback = `Request failed: ${res.status}`;
    try {
        const data = (await res.json()) as unknown;
        return new APIError(res.status, extractErrorMessage(data) ?? fallback, extractErrorCode(data));
    } catch {
        return new APIError(res.status, fallback);
    }
}

export function extractErrorMessage(data: unknown): string | null {
    if (data == null) return null;
    if (typeof data === "string") return data;
    if (typeof data !== "object") return String(data);
    const obj = data as Record<string, unknown>;
    const directKeys = ["error", "message", "detail", "error_message"] as const;
    for (const key of directKeys) {
        const v = obj[key];
        if (typeof v === "string" && v.trim()) return v;
        if (v && typeof v === "object") {
            const nested = extractErrorMessage(v);
            if (nested) return nested;
        }
    }
    if (Array.isArray(obj.errors)) {
        const parts = obj.errors.map((e) => extractErrorMessage(e)).filter((m): m is string => Boolean(m));
        if (parts.length > 0) return parts.join("; ");
    }
    try {
        return JSON.stringify(data);
    } catch {
        return null;
    }
}

/** The backend's `{ error: { code } }`, e.g. `RATE_LIMITED`; null for any other body shape. */
function extractErrorCode(data: unknown): string | null {
    if (!data || typeof data !== "object") return null;
    const error = (data as { error?: unknown }).error;
    if (!error || typeof error !== "object") return null;
    const code = (error as { code?: unknown }).code;
    return typeof code === "string" && code ? code : null;
}

/** What a failed request means, classified for a person rather than a log. */
export type FailureKind =
    /** The browser reports no network at all. */
    | "offline"
    /** The request left but no server answered: ours is down, restarting, or the proxy in front of it gave up (502/504). */
    | "unreachable"
    /** The request was aborted or timed out waiting for an answer. */
    | "timeout"
    /** The backend answered with a non-2xx status. */
    | "http"
    /** Something else threw; `message` is all we have. */
    | "unknown";

export interface IFailure {
    kind: FailureKind;
    status: number | null;
    code: string | null;
    message: string;
}

const BROWSER_NETWORK_FAILURE = /failed to fetch|load failed|networkerror when attempting|network request failed/i;

/**
 * Classify anything a mutation can throw, on either side of the server-function
 * boundary. Duck-typed on `kind`, see `BackendErrorKind`. A `fetch` that fails
 * in the browser itself (our SSR host unreachable) throws a bare `TypeError`
 * whose message differs per engine: Chrome "Failed to fetch", Safari "Load
 * failed", Firefox "NetworkError when attempting to fetch resource.".
 */
export function classifyFailure(err: unknown): IFailure {
    const message = err instanceof Error ? err.message : typeof err === "string" ? err : err ? String(err) : "";
    if (typeof navigator !== "undefined" && navigator.onLine === false) {
        return { kind: "offline", status: null, code: null, message };
    }
    const e = err && typeof err === "object" ? (err as { kind?: unknown; status?: unknown; code?: unknown; name?: unknown }) : null;
    const code = e && typeof e.code === "string" ? e.code : null;
    if (e?.kind === "unreachable") {
        return { kind: code === "TIMEOUT" ? "timeout" : "unreachable", status: null, code, message };
    }
    if (e?.kind === "http" && typeof e.status === "number") {
        // A reverse proxy answering for a dead upstream is not the backend
        // speaking; treat it as the backend being gone.
        if (e.status === 502 || e.status === 504) return { kind: "unreachable", status: e.status, code, message };
        return { kind: "http", status: e.status, code, message };
    }
    if (e?.name === "AbortError" || e?.name === "TimeoutError") {
        return { kind: "timeout", status: null, code: null, message };
    }
    if (err instanceof TypeError && BROWSER_NETWORK_FAILURE.test(message)) {
        return { kind: "unreachable", status: null, code: null, message };
    }
    return { kind: "unknown", status: null, code: null, message };
}
