import { describe, expect, it } from "vitest";
import { sanitizePlainName, truncateCodePoints } from "./sanitize-input";

describe("truncateCodePoints", () => {
    it("leaves a short string alone", () => {
        expect(truncateCodePoints("abc", 5)).toBe("abc");
        expect(truncateCodePoints("😀😀", 2)).toBe("😀😀");
    });

    it("counts an astral character as one, never splitting its surrogate pair", () => {
        expect(truncateCodePoints("😀😀😀", 2)).toBe("😀😀");
        expect(truncateCodePoints("a😀b", 2)).toBe("a😀");
        expect(truncateCodePoints("ab😀", 3)).toBe("ab😀");
    });

    it("cuts plain text at the limit", () => {
        expect(truncateCodePoints("abcdef", 3)).toBe("abc");
    });
});

describe("sanitizePlainName", () => {
    it("caps by code points", () => {
        expect(sanitizePlainName("😀".repeat(5), 3)).toBe("😀😀😀");
    });
});
