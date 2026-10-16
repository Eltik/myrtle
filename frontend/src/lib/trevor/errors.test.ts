import { describe, expect, it } from "vitest";
import { APIError, BackendUnreachableError } from "#/lib/api/_shared";
import { trevorFailureKind } from "./errors";

/** What an APIError looks like after seroval: the class is gone, the own fields remain. */
function crossed(status: number, code: string | null = null): Error {
    return Object.assign(new Error("x"), { kind: "http", status, code });
}

describe("trevorFailureKind", () => {
    it.each([
        [401, "signedOut"],
        [403, "verify"],
        [409, "tooMany"],
        [413, "tooLong"],
        [429, "rateLimited"],
        [503, "busy"],
        [500, "other"],
        [400, "other"],
    ] as const)("maps %i to %s, before and after the server-function boundary", (status, kind) => {
        expect(trevorFailureKind(new APIError(status, "x"))).toBe(kind);
        expect(trevorFailureKind(crossed(status))).toBe(kind);
    });

    it("reads a RATE_LIMITED code on another status as rate limited", () => {
        expect(trevorFailureKind(crossed(400, "RATE_LIMITED"))).toBe("rateLimited");
    });

    it("leaves network failures to the site's describer", () => {
        expect(trevorFailureKind(new BackendUnreachableError(new Error("ECONNREFUSED")))).toBe("other");
        expect(trevorFailureKind(new TypeError("Failed to fetch"))).toBe("other");
        // A dead upstream behind the proxy is not Trevor being busy.
        expect(trevorFailureKind(crossed(502))).toBe("other");
    });
});
