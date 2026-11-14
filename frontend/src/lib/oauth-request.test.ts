import { describe, expect, it } from "vitest";
import { packAuthorizeRequest, unpackAuthorizeRequest } from "./oauth-request";

describe("the authorization request survives the trip to the consent page", () => {
    it("returns every parameter byte for byte", () => {
        // Values the router's JSON search parser would rewrite: a number in
        // exponent form, leading zeros, a boolean, an object, a long integer.
        for (const state of ["1e3", "007", "true", "null", '{"a":1}', "12345678901234567890", "st&te=1 +x", "日本語"]) {
            const search = `?response_type=code&client_id=abc&state=${encodeURIComponent(state)}&redirect_uri=${encodeURIComponent("https://client.example/cb?x=1")}`;
            const request = unpackAuthorizeRequest(packAuthorizeRequest(search));
            expect(request.state).toBe(state);
            expect(request.redirect_uri).toBe("https://client.example/cb?x=1");
            expect(request.client_id).toBe("abc");
        }
    });

    it("packs to a value no JSON parser accepts and no URL needs to escape", () => {
        for (const search of ["", "?", "?state=1", `?a=%FF&state=${"9".repeat(500)}`]) {
            const packed = packAuthorizeRequest(search);
            expect(packed).toMatch(/^q\.[A-Za-z0-9_-]*$/);
            expect(() => JSON.parse(packed)).toThrow();
            expect(encodeURIComponent(packed)).toBe(packed);
        }
    });

    it("keeps only the parameters of an authorization request", () => {
        const request = unpackAuthorizeRequest(packAuthorizeRequest("?client_id=abc&approve=true&next=/admin"));
        expect(request).toEqual({ client_id: "abc" });
    });

    it("reads anything that is not a packed request as an empty one", () => {
        for (const bad of [undefined, null, 1000, true, "", "abc", "q.%%%", { request: "q.x" }]) {
            expect(unpackAuthorizeRequest(bad)).toEqual({});
        }
    });
});
