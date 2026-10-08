import { describe, expect, it } from "vitest";
import { DEFAULT_SORT, defaultDir, isMetricSort, MAX_HAS, parseAll, parseDir, parseOperatorId, parseOperatorIds, parseScope, parseSort, scopeToken, splitOperatorIds } from "./searchControls";

describe("parseScope and scopeToken", () => {
    it("reads class and archetype scopes and writes them back", () => {
        expect(parseScope("class:WARRIOR")).toEqual({ kind: "class", profession: "WARRIOR" });
        expect(parseScope("sub:centurion")).toEqual({ kind: "sub", subProfessionId: "centurion" });
        expect(scopeToken({ kind: "class", profession: "SNIPER" })).toBe("class:SNIPER");
        expect(scopeToken({ kind: "sub", subProfessionId: "fastshot" })).toBe("sub:fastshot");
    });

    it("rejects unknown classes, malformed archetypes and extra segments", () => {
        for (const token of ["class:TOKEN", "class:warrior", "sub:Centurion", "sub:a-b", "sub:", "class:", "sub:a:b", "team:x", "score", "", 42, null, undefined]) {
            expect(parseScope(token)).toBeNull();
        }
    });
});

describe("parseSort, parseAll and parseDir", () => {
    it("keeps metric and scoped sorts and falls back to score", () => {
        expect(parseSort("masteries")).toBe("masteries");
        expect(parseSort("class:CASTER")).toBe("class:CASTER");
        expect(parseSort("sub:corecaster")).toBe("sub:corecaster");
        expect(parseSort("bogus")).toBe(DEFAULT_SORT);
        expect(parseSort(undefined)).toBe("score");
        expect(parseSort(["score"])).toBe("score");
    });

    it("keeps only a valid scope as the all filter", () => {
        expect(parseAll("class:MEDIC")).toBe("class:MEDIC");
        expect(parseAll("masteries")).toBe("");
        expect(parseAll(undefined)).toBe("");
    });

    it("accepts only asc and desc", () => {
        expect(parseDir("asc")).toBe("asc");
        expect(parseDir("desc")).toBe("desc");
        expect(parseDir("ASC")).toBeUndefined();
        expect(parseDir(1)).toBeUndefined();
    });

    it("sorts joined oldest first and every count high to low", () => {
        expect(defaultDir("joined")).toBe("asc");
        expect(defaultDir("score")).toBe("desc");
        expect(defaultDir("class:WARRIOR")).toBe("desc");
    });

    it("knows the metric sorts", () => {
        expect(isMetricSort("skins")).toBe(true);
        expect(isMetricSort("class:WARRIOR")).toBe(false);
    });
});

describe("operator ids", () => {
    it("accepts word characters in either case", () => {
        expect(parseOperatorId("char_002_amiya")).toBe("char_002_amiya");
        expect(parseOperatorId("CHAR_002")).toBe("CHAR_002");
        expect(parseOperatorId("char-002")).toBe("");
        expect(parseOperatorId("")).toBe("");
        expect(parseOperatorId(7)).toBe("");
    });

    it("has no length cap, unlike the backend's 50-character check", () => {
        expect(parseOperatorId("a".repeat(60))).toBe("a".repeat(60));
    });

    it("splits, trims, drops malformed ids and dedupes in first-seen order", () => {
        expect(splitOperatorIds(" char_a , bad id,char_b,,char_a ")).toEqual(["char_a", "char_b"]);
        expect(parseOperatorIds("char_b,char_a,char_b")).toBe("char_b,char_a");
        expect(splitOperatorIds("")).toEqual([]);
        expect(splitOperatorIds(undefined)).toEqual([]);
    });

    it("stops at the backend's MAX_HAS", () => {
        const ids = Array.from({ length: MAX_HAS + 5 }, (_, i) => `char_${i}`);
        const kept = splitOperatorIds(ids.join(","));
        expect(MAX_HAS).toBe(20);
        expect(kept).toHaveLength(MAX_HAS);
        expect(kept[MAX_HAS - 1]).toBe("char_19");
    });
});
