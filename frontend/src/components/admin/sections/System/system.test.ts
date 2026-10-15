import { describe, expect, it } from "vitest";
import { changeKind } from "#/components/admin/shell/model";
import type { IAuditLogEntry, IHealthResponse } from "#/lib/api/admin";
import type { Locale } from "#/types/generated/Locale";
import type { TranslationAuditEntry } from "#/types/generated/TranslationAuditEntry";
import { buildTimeline, filterTimeline, previewValue } from "./audit";
import { healthDot, summarizeHealth } from "./health";
import { emptyLocaleForm, fallbackCycles, localeForm, NO_FALLBACK, nextSortOrder, sortLocales, toggleInput, translatedPercent, upsertInput, validateLocaleForm } from "./locales";

const actor = { user_id: "u1", uid: "123", nickname: "shirayuki", secretary: null, secretary_skin_id: null };

function note(id: number, at: string, over: Partial<IAuditLogEntry> = {}): IAuditLogEntry {
    return { id, note_id: "n", operator_id: "char_350_surtr", field_name: "pros", old_value: null, new_value: "Burns things", changed_at: at, actor, ...over };
}

function tr(id: number, at: string, over: Partial<TranslationAuditEntry> = {}): TranslationAuditEntry {
    return { id, message_key: "home.hero.cta", locale: "ja", old_value: null, new_value: "検索", changed_at: at, actor, ...over };
}

function locale(code: string, over: Partial<Locale> = {}): Locale {
    return { code, english_name: code, native_name: code, fallback_locale: null, gamedata_server: "en", enabled: true, sort_order: 10, is_source: false, ...over };
}

describe("changeKind / previewValue", () => {
    it("classifies blank-to-text as added, text-to-blank as cleared, anything else as changed", () => {
        expect(changeKind(null, "x")).toBe("added");
        expect(changeKind("  ", "x")).toBe("added");
        expect(changeKind("x", null)).toBe("cleared");
        expect(changeKind("x", "")).toBe("cleared");
        expect(changeKind("x", "y")).toBe("changed");
    });

    it("previews the new value on one line, or the old one for a clear, cut to 80 characters", () => {
        expect(previewValue(null, "a\n\nb")).toBe("a b");
        expect(previewValue("gone", null)).toBe("gone");
        const long = "x".repeat(100);
        expect(previewValue(null, long)).toBe(`${"x".repeat(80)}…`);
    });
});

describe("buildTimeline", () => {
    it("merges both logs newest first with keys that cannot collide", () => {
        const { rows, held } = buildTimeline({
            notes: [note(1, "2026-10-09T10:00:00Z"), note(2, "2026-10-07T10:00:00Z")],
            translations: [tr(1, "2026-10-08T10:00:00Z")],
            notesHasMore: false,
            translationsHasMore: false,
            source: "all",
        });
        expect(rows.map((r) => r.key)).toEqual(["n1", "t1", "n2"]);
        expect(rows[1]).toMatchObject({ source: "translations", field: "text", locale: "ja", messageKey: "home.hero.cta" });
        expect(held).toBe(0);
    });

    it("holds back rows older than the oldest loaded row of a log that has more pages", () => {
        const { rows, held } = buildTimeline({
            notes: [note(1, "2026-10-09T10:00:00Z"), note(2, "2026-10-08T00:00:00Z")],
            translations: [tr(1, "2026-10-08T12:00:00Z"), tr(2, "2026-10-01T00:00:00Z")],
            notesHasMore: true,
            translationsHasMore: false,
            source: "all",
        });
        expect(rows.map((r) => r.key)).toEqual(["n1", "t1", "n2"]);
        expect(held).toBe(1);
    });

    it("ignores the other log, and its paging, when filtered to one source", () => {
        const { rows, held } = buildTimeline({
            notes: [note(1, "2026-10-09T10:00:00Z")],
            translations: [tr(1, "2026-10-01T00:00:00Z")],
            notesHasMore: true,
            translationsHasMore: false,
            source: "translations",
        });
        expect(rows.map((r) => r.key)).toEqual(["t1"]);
        expect(held).toBe(0);
    });
});

describe("filterTimeline", () => {
    const { rows } = buildTimeline({ notes: [note(1, "2026-10-09T10:00:00Z", { field_name: "trivia" })], translations: [tr(1, "2026-10-08T10:00:00Z")], notesHasMore: false, translationsHasMore: false, source: "all" });

    it("filters by field, with text meaning translations", () => {
        expect(filterTimeline(rows, "trivia", "", () => []).map((r) => r.key)).toEqual(["n1"]);
        expect(filterTimeline(rows, "text", "", () => []).map((r) => r.key)).toEqual(["t1"]);
    });

    it("matches free text against values, keys and the supplied display labels, ignoring case and punctuation", () => {
        expect(filterTimeline(rows, "all", "HOME.hero", () => []).map((r) => r.key)).toEqual(["t1"]);
        expect(filterTimeline(rows, "all", "surtr", (r) => (r.source === "notes" ? ["Surtr"] : ["Japanese"])).map((r) => r.key)).toEqual(["n1"]);
        expect(filterTimeline(rows, "all", "japanese", (r) => (r.source === "notes" ? ["Surtr"] : ["Japanese"])).map((r) => r.key)).toEqual(["t1"]);
    });
});

describe("summarizeHealth", () => {
    const base: IHealthResponse = { status: "ok", cache: { backend: "redis", status: "connected", responseTimeMs: 1 }, database: { status: "connected", responseTimeMs: 4 }, timestamp: "", responseTimeMs: 30 };

    it("tells a pending probe from a failed one", () => {
        expect(summarizeHealth(undefined, false)).toBe("checking");
        expect(summarizeHealth(undefined, true)).toBe("unreachable");
    });

    it("names which service is down, and keeps a cache outage amber", () => {
        expect(summarizeHealth(base, false)).toBe("healthy");
        expect(summarizeHealth({ ...base, status: "degraded", cache: { ...base.cache, status: "disconnected" } }, false)).toBe("cacheDown");
        expect(summarizeHealth({ ...base, status: "degraded", database: { ...base.database, status: "disconnected" } }, false)).toBe("databaseDown");
        expect(summarizeHealth({ ...base, status: "degraded", database: { ...base.database, status: "disconnected" }, cache: { ...base.cache, status: "disconnected" } }, false)).toBe("bothDown");
        expect(summarizeHealth({ ...base, status: "degraded" }, false)).toBe("degraded");
        expect(healthDot("healthy")).toBe("green");
        expect(healthDot("cacheDown")).toBe("amber");
        expect(healthDot("databaseDown")).toBe("red");
    });
});

describe("locale form", () => {
    const locales = [locale("en", { is_source: true, sort_order: 0 }), locale("zh-CN", { sort_order: 30 }), locale("zh-TW", { fallback_locale: "zh-CN", sort_order: 40 })];

    it("detects a fallback chain that loops back", () => {
        expect(fallbackCycles("zh-CN", "zh-TW", locales)).toBe(true);
        expect(fallbackCycles("ja", "zh-TW", locales)).toBe(false);
        const loop = [locale("a", { fallback_locale: "b" }), locale("b", { fallback_locale: "a" })];
        expect(fallbackCycles("c", "a", loop)).toBe(true);
    });

    it("refuses bad codes, duplicate codes, blank names, cycles and bad sort orders", () => {
        expect(validateLocaleForm({ ...emptyLocaleForm(50) }, locales, true)).toMatchObject({ code: "codeRequired", englishName: "englishNameRequired", nativeName: "nativeNameRequired" });
        expect(validateLocaleForm({ ...emptyLocaleForm(50), code: "Japanese" }, locales, true).code).toBe("codeShape");
        expect(validateLocaleForm({ ...emptyLocaleForm(50), code: "zh-CN" }, locales, true).code).toBe("codeExists");
        expect(validateLocaleForm({ ...localeForm(locales[1]), fallbackLocale: "zh-TW" }, locales, false).fallbackLocale).toBe("fallbackCycle");
        expect(validateLocaleForm({ ...localeForm(locales[1]), fallbackLocale: "zh-CN" }, locales, false).fallbackLocale).toBe("fallbackSelf");
        expect(validateLocaleForm({ ...localeForm(locales[0]), fallbackLocale: "zh-CN" }, locales, false).fallbackLocale).toBe("fallbackSource");
        expect(validateLocaleForm({ ...localeForm(locales[1]), sortOrder: "-1" }, locales, false).sortOrder).toBe("sortOrder");
        expect(validateLocaleForm({ ...localeForm(locales[2]) }, locales, false)).toEqual({});
    });

    it("builds upsert bodies that keep the row and change only what was asked", () => {
        expect(upsertInput({ ...emptyLocaleForm(50), code: " ja ", englishName: "Japanese", nativeName: "日本語", gamedataServer: "jp" }, false)).toEqual({ code: "ja", englishName: "Japanese", nativeName: "日本語", fallbackLocale: null, gamedataServer: "jp", enabled: false, sortOrder: 50 });
        expect(toggleInput(locales[2], false)).toMatchObject({ code: "zh-TW", fallbackLocale: "zh-CN", enabled: false, sortOrder: 40 });
        expect(localeForm(locales[0]).fallbackLocale).toBe(NO_FALLBACK);
    });

    it("orders the source first and appends new languages after the last", () => {
        expect(sortLocales([locales[2], locales[0], locales[1]]).map((l) => l.code)).toEqual(["en", "zh-CN", "zh-TW"]);
        expect(nextSortOrder(locales)).toBe(50);
        expect(nextSortOrder([])).toBe(0);
    });

    it("counts only current translations and never rounds up to complete", () => {
        expect(translatedPercent({ locale: "ja", total: 1000, translated: 1000, stale: 1 })).toBe(99);
        expect(translatedPercent({ locale: "ja", total: 100, translated: 29, stale: 0 })).toBe(29);
        expect(translatedPercent({ locale: "ja", total: 200, translated: 100, stale: 0 })).toBe(50);
        expect(translatedPercent({ locale: "ja", total: 0, translated: 0, stale: 0 })).toBeNull();
        expect(translatedPercent(undefined)).toBeNull();
    });
});
