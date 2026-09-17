import { classifyFailure, type IFailure } from "#/lib/api/_shared";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./error-message.messages";

/** `parseError`'s fallback when the response body had no readable message. */
const BARE_STATUS = /^Request failed: \d+$/;

/**
 * Turn whatever a mutation threw into the sentence an error toast shows.
 *
 * Names the situation first (offline, server unreachable, timed out, rate
 * limited, server error) and only falls back to the server's own message for
 * a 4xx that carries one, which is where the specific reason lives (a wrong
 * verification code comes back as Yostar's text, for instance). Works with the
 * bundled English catalog alone, so it still renders while the backend is down,
 * which is precisely when it is needed.
 */
export function useErrorMessage(): (err: unknown) => string {
    const t: TypedT<typeof messages> = useT("common");
    return (err: unknown) => describe(classifyFailure(err), t);
}

function describe(f: IFailure, t: TypedT<typeof messages>): string {
    switch (f.kind) {
        case "offline":
            return t("errorMessage.offline");
        case "unreachable":
            return f.code ? t("errorMessage.unreachableWithCode", { code: f.code }) : t("errorMessage.unreachable");
        case "timeout":
            return t("errorMessage.timeout");
        case "http":
            return describeHttp(f, t);
        case "unknown":
            return f.message.trim() || t("errorMessage.unknown");
    }
}

function describeHttp(f: IFailure, t: TypedT<typeof messages>): string {
    const status = f.status ?? 0;
    // `code` is the backend's `ApiError` discriminator (backend/src/app/error.rs);
    // `SYNC_FAILED` is ours, minted in `lib/auth/server.ts` `completeLogin`.
    if (f.code === "SYNC_FAILED") return t("errorMessage.syncFailed", { status });
    if (status === 429 || f.code === "RATE_LIMITED") return t("errorMessage.rateLimited");
    if (status === 503 || f.code === "SERVICE_UNAVAILABLE") return t("errorMessage.unavailable", { status });
    if (status >= 500) return t("errorMessage.serverError", { status });
    if (status === 401 || status === 403) return t("errorMessage.unauthorized", { status });
    const message = f.message.trim();
    if (!message || BARE_STATUS.test(message)) return t("errorMessage.rejected", { status });
    return message;
}
