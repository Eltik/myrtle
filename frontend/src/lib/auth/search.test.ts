import { defaultParseSearch, defaultStringifySearch } from "@tanstack/react-router";
import { describe, expect, it } from "vitest";
import { isSignInPrompt, SIGN_IN_PROMPT, signInSearch } from "./search";

/** What the router hands every route as `location.search` for a query string. */
const parse = (query: string) => defaultParseSearch(query) as Record<string, unknown>;

describe("sign-in deep link", () => {
    it.each([
        ["?auth=1", "typed by hand"],
        ["?auth=%221%22", "what the redirects emitted before"],
        ["?auth=true", "boolean spelling"],
    ])("%s (%s) opens the prompt after the router parses it", (query) => {
        expect(isSignInPrompt(parse(query).auth)).toBe(true);
    });

    it.each(["?auth=0", "?auth=%220%22", "?auth=false", "?auth=yes", "?other=1", ""])("%s does not", (query) => {
        expect(isSignInPrompt(parse(query).auth)).toBe(false);
    });

    it("redirects serialize as the clean ?auth=1 and round-trip", () => {
        const query = defaultStringifySearch(signInSearch("/tier-lists/my?sort=recent"));
        expect(query).toBe("?auth=1&next=%2Ftier-lists%2Fmy%3Fsort%3Drecent");
        const parsed = parse(query);
        expect(parsed.auth).toBe(SIGN_IN_PROMPT);
        expect(parsed.next).toBe("/tier-lists/my?sort=recent");
    });
});
