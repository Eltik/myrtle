import { describe, expect, it } from "vitest";
import { describeMessage } from "#/lib/i18n/format";
import { entryStatus, flattenPages, formRows, hasPlaceholder, keyAfterSave, keyAfterSkip, localeStats, mergeArguments, missingPlaceholders, placeholderNames, resolveFilter, resolveLocale, selectEntry, sourceArguments } from "./model";

const rows = (...keys: string[]) => keys.map((key) => ({ key }));

describe("entryStatus", () => {
    it("reads untranslated, stale and translated", () => {
        expect(entryStatus({ value: null, is_stale: false })).toBe("untranslated");
        expect(entryStatus({ value: "x", is_stale: true })).toBe("stale");
        expect(entryStatus({ value: "x", is_stale: false })).toBe("translated");
    });
});

describe("localeStats", () => {
    it("counts stale strings as not done and floors to one decimal", () => {
        expect(localeStats({ total: 2480, translated: 2400, stale: 30 })).toEqual({ total: 2480, stale: 30, missing: 80, todo: 110, pct: 95.5 });
    });

    it("treats an empty catalog as complete and a missing row as nothing", () => {
        expect(localeStats({ total: 0, translated: 0, stale: 0 }).pct).toBe(100);
        expect(localeStats(undefined)).toEqual({ total: 0, stale: 0, missing: 0, todo: 0, pct: 0 });
    });
});

describe("resolveFilter / resolveLocale", () => {
    it("defaults to the backlog for editors and to everything for viewers", () => {
        expect(resolveFilter(undefined, true)).toBe("todo");
        expect(resolveFilter(undefined, false)).toBe("all");
        expect(resolveFilter("stale", false)).toBe("stale");
    });

    it("keeps a visible locale and falls back to the first", () => {
        expect(resolveLocale(["ja", "ko"], "ko")).toBe("ko");
        expect(resolveLocale(["ja", "ko"], "zh-TW")).toBe("ja");
        expect(resolveLocale([], "ja")).toBeUndefined();
    });
});

describe("placeholders", () => {
    it("reads arrays and tolerates the object shape", () => {
        expect(placeholderNames(["a", 1, "b"])).toEqual(["a", "b"]);
        expect(placeholderNames({ a: 1 })).toEqual(["a"]);
        expect(placeholderNames(null)).toEqual([]);
    });

    it("finds plain and formatted arguments", () => {
        expect(hasPlaceholder("{title} を公開", "title")).toBe(true);
        expect(hasPlaceholder("{ count, plural, other {# 件} }", "count")).toBe(true);
        expect(hasPlaceholder("{titles}", "title")).toBe(false);
        expect(hasPlaceholder("title", "title")).toBe(false);
        expect(missingPlaceholders(["count", "when"], "{count} 理性")).toEqual(["when"]);
    });

    it("does not mistake words inside plural branches for arguments", () => {
        const names = sourceArguments("{count, plural, one {You have # item} other {You have # items}}", []).map((a) => a.name);
        expect(names).toEqual(["count"]);
    });

    it("keeps declared arguments the parser did not reach", () => {
        const merged = mergeArguments(describeMessage("Hi {name}"), ["name", "extra"]);
        expect(merged.map((a) => [a.name, a.type])).toEqual([
            ["name", "plain"],
            ["extra", "plain"],
        ]);
    });
});

describe("list navigation", () => {
    it("selects the explicit key or the first row", () => {
        expect(selectEntry(rows("a", "b"), "b")?.key).toBe("b");
        expect(selectEntry(rows("a", "b"), "zzz")?.key).toBe("a");
        expect(selectEntry([], "a")).toBeUndefined();
    });

    it("saves forward, or back from the last row", () => {
        expect(keyAfterSave(rows("a", "b", "c"), "a")).toBe("b");
        expect(keyAfterSave(rows("a", "b", "c"), "c")).toBe("b");
        expect(keyAfterSave(rows("a"), "a")).toBeUndefined();
    });

    it("skips forward and wraps", () => {
        expect(keyAfterSkip(rows("a", "b"), "a")).toBe("b");
        expect(keyAfterSkip(rows("a", "b"), "b")).toBe("a");
        expect(keyAfterSkip([], "a")).toBeUndefined();
    });

    it("drops a row repeated at a page seam", () => {
        expect(flattenPages([{ entries: rows("a", "b") }, { entries: rows("b", "c") }]).map((e) => e.key)).toEqual(["a", "b", "c"]);
    });
});

describe("formRows", () => {
    const [arg] = describeMessage("{count, plural, one {# day} other {# days}}");

    it("asks for the target locale's forms, not the English ones", () => {
        const ru = formRows(arg, null, "ru", false).map((r) => r.key);
        expect(ru).toEqual(["one", "few", "many", "other"]);
    });

    it("flags a required form only once the translator has started", () => {
        const [written] = describeMessage("{count, plural, one {# день} other {# дней}}");
        const started = formRows(arg, written, "ru", true);
        expect(started.find((r) => r.key === "few")?.status).toBe("missing");
        expect(started.find((r) => r.key === "one")?.status).toBe("written");
        expect(formRows(arg, null, "ru", false).every((r) => r.status === "unused")).toBe(true);
    });

    it("keeps the English wording and an exact form's value", () => {
        const [exact] = describeMessage("{count, plural, =0 {none} one {# day} other {# days}}");
        const table = formRows(exact, null, "en", false);
        expect(table[0]).toEqual({ key: "=0", applies: "0", english: "none", status: "unused" });
        expect(table.find((r) => r.key === "other")?.applies).toBe("");
    });
});
