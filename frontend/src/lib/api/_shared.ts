export interface IBackendStatus {
    status: string;
}

export class ApiError extends Error {
    constructor(
        public readonly status: number,
        message: string,
    ) {
        super(message);
        this.name = "ApiError";
    }
}

export async function parseError(res: Response): Promise<ApiError> {
    const fallback = `Request failed: ${res.status}`;
    try {
        const data = (await res.json()) as unknown;
        return new ApiError(res.status, extractErrorMessage(data) ?? fallback);
    } catch {
        return new ApiError(res.status, fallback);
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
