import { describe, expect, it } from "vitest";
import { emptyOperators } from "./inbox";
import { bestLocaleGrants, computeSectionAccess, roleMayReachSection, sectionForPath, translationsTodo } from "./model";
import { parseSystemSearch, parseTranslationsSearch, parseUsersSearch } from "./search";

describe("computeSectionAccess", () => {
    it("gives staff every section but Translations, which only a super-admin gets without grants", () => {
        expect(computeSectionAccess("tier_list_admin", 0, 0)).toEqual({ home: true, users: true, system: true, notes: true, tierlists: true, translations: false });
        expect(computeSectionAccess("super_admin", 0, 0).translations).toBe(true);
    });

    it("scopes an editor to notes plus granted lists", () => {
        expect(computeSectionAccess("tier_list_editor", 0, 0)).toEqual({ home: true, users: false, system: false, notes: true, tierlists: false, translations: false });
        expect(computeSectionAccess("tier_list_editor", 2, 0).tierlists).toBe(true);
    });

    it("opens Translations to any role once it holds a locale grant", () => {
        expect(computeSectionAccess("translator", 0, 0).translations).toBe(false);
        expect(computeSectionAccess("translator", 0, 1)).toEqual({ home: true, users: false, system: false, notes: false, tierlists: false, translations: true });
        // Real holders: tier-list admins, editors and plain users, none of them `translator`.
        for (const role of ["tier_list_admin", "tier_list_editor", "user"]) {
            expect(computeSectionAccess(role, 0, 0).translations).toBe(false);
            expect(computeSectionAccess(role, 0, 1).translations).toBe(true);
        }
    });
});

describe("bestLocaleGrants", () => {
    it("keeps one grant per locale at the highest level and drops unknown levels", () => {
        const rows = [
            { locale: "ko", permission: "view" },
            { locale: "ja", permission: "edit" },
            { locale: "ja", permission: "admin" },
            { locale: "ja", permission: "view" },
            { locale: "zh", permission: "owner" },
        ];
        expect(bestLocaleGrants(rows)).toEqual([
            { code: "ja", level: "admin" },
            { code: "ko", level: "view" },
        ]);
        expect(bestLocaleGrants([])).toEqual([]);
    });
});

describe("roleMayReachSection", () => {
    it("redirects roles that can never reach a section and lets grant-dependent ones through", () => {
        expect(roleMayReachSection("users", "tier_list_editor")).toBe(false);
        expect(roleMayReachSection("system", "tier_list_admin")).toBe(true);
        expect(roleMayReachSection("notes", "translator")).toBe(false);
        // Translations is grant-gated, never role-gated: the section itself sends a grantless user home.
        for (const role of ["super_admin", "tier_list_admin", "tier_list_editor", "translator", "user", null]) expect(roleMayReachSection("translations", role)).toBe(true);
        expect(roleMayReachSection("tierlists", "translator")).toBe(true);
    });
});

describe("sectionForPath", () => {
    it("maps section URLs, trailing slashes and unknown admin paths", () => {
        expect(sectionForPath("/admin")).toBe("home");
        expect(sectionForPath("/admin/")).toBe("home");
        expect(sectionForPath("/admin/tier-lists")).toBe("tierlists");
        expect(sectionForPath("/admin/operator-notes/")).toBe("notes");
        expect(sectionForPath("/admin/nope")).toBe("home");
    });
});

describe("counts", () => {
    it("counts index operators without a filled note, ignoring notes for unknown ids", () => {
        expect(
            emptyOperators(
                ["a", "b", "c", "d"].map((id) => ({ id, name: id, rarity: 6 })),
                new Set(["a", "c", "zz"]),
            ).length,
        ).toBe(2);
    });

    it("sums stale + untranslated over editable locales, public ones only for a super-admin", () => {
        const progress = [
            { locale: "ja", total: 10, translated: 8, stale: 1 },
            { locale: "ko", total: 10, translated: 5, stale: 0 },
        ];
        const translator = {
            isSuper: false,
            myLoc: [
                { code: "ja", level: "edit" as const },
                { code: "ko", level: "view" as const },
            ],
        };
        expect(translationsTodo(translator, progress, new Set())).toBe(3);
        const superAdmin = {
            isSuper: true,
            myLoc: [
                { code: "ja", level: "admin" as const },
                { code: "ko", level: "admin" as const },
            ],
        };
        expect(translationsTodo(superAdmin, progress, new Set(["ko"]))).toBe(5);
    });
});

describe("search parsers", () => {
    it("keeps known values and drops unknown or empty ones", () => {
        expect(parseUsersSearch({ user: "u1", filter: "staff", q: "", bogus: 1 })).toEqual({ user: "u1", filter: "staff" });
        expect(parseTranslationsSearch({ filter: "nope", locale: "ja" })).toEqual({ locale: "ja" });
        expect(parseSystemSearch({ tab: "audit" })).toEqual({ tab: "audit" });
    });
});
