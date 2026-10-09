import { describe, expect, it } from "vitest";
import { orphanLocaleGrants } from "#/components/admin/shell/inbox";
import { accessSummary, accessWarning, holderOf, type IPersonLocaleGrant, type IPersonTierListGrant, initialsOf, isDeletedAccount, parsePeopleServer, personLabel, pickerForWarning } from "./warnings";

const tl = (userId: string, slug: string, permission = "edit", nickname: string | null = null): IPersonTierListGrant => ({
    slug,
    title: slug.toUpperCase(),
    permission,
    userId,
    userUid: `uid-${userId}`,
    userNickname: nickname,
    grantedBy: null,
    grantedByNickname: null,
    grantedAt: "2026-10-01T00:00:00Z",
});
const loc = (userId: string, locale: string, permission = "edit", nickname: string | null = null): IPersonLocaleGrant => ({
    locale,
    permission,
    user_id: userId,
    user_uid: `uid-${userId}`,
    user_nickname: nickname,
    granted_by: null,
    granted_by_nickname: null,
    granted_at: "2026-10-01T00:00:00Z",
});
const none = { tierLists: [], locales: [] };

describe("accessWarning", () => {
    it("flags a translator whose language grants are all View", () => {
        expect(accessWarning("translator", { tierLists: [], locales: [loc("a", "ja", "view")] })).toBe("translatorNoEdit");
        expect(accessWarning("translator", none)).toBe("translatorNoEdit");
        expect(accessWarning("translator", { tierLists: [], locales: [loc("a", "ja", "view"), loc("a", "ko", "edit")] })).toBeNull();
    });

    it("accepts an editor with no lists and a Player holding grants", () => {
        expect(accessWarning("tier_list_editor", none)).toBeNull();
        expect(accessWarning("user", { tierLists: [], locales: [loc("a", "ja", "edit")] })).toBeNull();
        expect(accessWarning("user", { tierLists: [tl("a", "x")], locales: [] })).toBeNull();
    });

    it("never flags the admin roles", () => {
        expect(accessWarning("tier_list_admin", none)).toBeNull();
        expect(accessWarning("super_admin", { tierLists: [tl("a", "x")], locales: [loc("a", "ja")] })).toBeNull();
    });
});

describe("accessSummary", () => {
    const name = (code: string) => ({ ja: "Japanese", ko: "Korean", fr: "French" })[code] ?? code;

    it("names the lists an editor holds and the languages a translator can edit", () => {
        expect(accessSummary("tier_list_editor", { tierLists: [tl("a", "meta")], locales: [] }, name)).toEqual({ kind: "editor", lists: ["META"], locales: [] });
        expect(accessSummary("tier_list_editor", none, name)).toEqual({ kind: "editor", lists: [], locales: [] });
        expect(accessSummary("translator", { tierLists: [], locales: [loc("a", "ja", "view"), loc("a", "ko", "publish")] }, name)).toEqual({ kind: "translator", locales: ["Korean"] });
    });

    it("adds editable languages to every role below super-admin", () => {
        const french = { tierLists: [], locales: [loc("a", "fr"), loc("a", "ja", "view")] };
        expect(accessSummary("tier_list_admin", french, name)).toEqual({ kind: "allLists", locales: ["French"] });
        expect(accessSummary("tier_list_editor", french, name)).toEqual({ kind: "editor", lists: [], locales: ["French"] });
        expect(accessSummary("user", french, name)).toEqual({ kind: "player", locales: ["French"] });
        expect(accessSummary("super_admin", french, name)).toEqual({ kind: "everything" });
    });

    it("gives a Player with no language grant nothing, whatever lists they hold", () => {
        expect(accessSummary("user", { tierLists: [tl("a", "x")], locales: [] }, name)).toEqual({ kind: "player", locales: [] });
    });
});

describe("deleted accounts", () => {
    const gone = { ...loc("ghost", "ko"), user_uid: null, user_nickname: null };

    it("finds the language grants whose account row is gone", () => {
        expect(orphanLocaleGrants([loc("a", "ja"), gone])).toEqual([gone]);
        expect(orphanLocaleGrants([loc("a", "ja")])).toEqual([]);
    });

    it("calls an id deleted only when it holds grants and all of them are orphaned", () => {
        expect(isDeletedAccount({ tierLists: [], locales: [gone] })).toBe(true);
        expect(isDeletedAccount(none)).toBe(false);
        expect(isDeletedAccount({ tierLists: [], locales: [loc("a", "ja")] })).toBe(false);
        expect(isDeletedAccount({ tierLists: [tl("ghost", "x")], locales: [gone] })).toBe(false);
    });
});

describe("holderOf", () => {
    it("names a holder from a tier-list grant first, else a language grant", () => {
        expect(holderOf({ tierLists: [tl("a", "x", "edit", "Amy")], locales: [loc("a", "ja", "edit", "Other")] })).toEqual({ nickname: "Amy", uid: "uid-a" });
        expect(holderOf({ tierLists: [], locales: [loc("a", "ja", "edit", "Amy")] })).toEqual({ nickname: "Amy", uid: "uid-a" });
        expect(holderOf({ tierLists: [], locales: [{ ...loc("a", "ja"), user_uid: null }] })).toEqual({ nickname: null, uid: null });
        expect(holderOf(none)).toEqual({ nickname: null, uid: null });
    });
});

describe("helpers", () => {
    it("labels a person by nickname, then UID, and never by a raw id", () => {
        expect(personLabel({ nickname: "Doc", uid: "9" }, "Deleted account")).toBe("Doc");
        expect(personLabel({ nickname: null, uid: "9" }, "Deleted account")).toBe("9");
        expect(personLabel({ nickname: null, uid: null }, "Deleted account")).toBe("Deleted account");
    });

    it("takes two initials, ignoring punctuation", () => {
        expect(initialsOf("blue poison")).toBe("BP");
        expect(initialsOf("Kal'tsit")).toBe("KT");
        expect(initialsOf("ドクター")).toBe("ド");
        expect(initialsOf("!!")).toBe("!!");
    });

    it("opens the picker that fixes the warning", () => {
        expect(pickerForWarning("translatorNoEdit")).toBe("locales");
        expect(pickerForWarning(null)).toBeNull();
    });

    it("accepts only the server codes the People tabs show", () => {
        expect(parsePeopleServer("jp")).toBe("jp");
        expect(parsePeopleServer("bili")).toBe("bili");
        expect(parsePeopleServer("tw")).toBe("tw");
        expect(parsePeopleServer("JP")).toBeUndefined();
        expect(parsePeopleServer(undefined)).toBeUndefined();
    });
});
