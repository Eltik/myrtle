import { describe, expect, it } from "vitest";
import { legacyRedirect } from "./legacy-routes";

describe("legacyRedirect", () => {
    it.each([
        ["/dps-calculator", "/tools/dps"],
        ["/recruitment-calculator", "/tools/recruitment"],
        ["/squad-randomizer", "/tools/randomizer"],
        ["/tools/enemies", "/enemies"],
        ["/collection/enemies", "/enemies"],
        ["/collection/operators", "/operators"],
        ["/tier-list", "/tier-lists"],
        ["/users/leaderboard", "/user/leaderboard"],
        ["/users/search", "/user/search"],
        ["/dps-calculator/", "/tools/dps"],
    ])("sends %s to %s", (from, to) => {
        expect(legacyRedirect(from, {})).toBe(to);
    });

    it("sends the old operator query to the operator's page", () => {
        expect(legacyRedirect("/operators", { id: "char_1014_nearl2" })).toBe("/operators/char_1014_nearl2");
        expect(legacyRedirect("/collection/operators", { id: "char_002_amiya" })).toBe("/operators/char_002_amiya");
    });

    it("leaves current pages alone", () => {
        for (const path of ["/", "/operators", "/tools/dps", "/enemies", "/user/leaderboard", "/operators/char_002_amiya"]) {
            expect(legacyRedirect(path, {})).toBeNull();
        }
        expect(legacyRedirect("/operators", { page: 2 })).toBeNull();
    });

    it("refuses an id that could leave the site", () => {
        for (const id of ["//evil.example", "../admin", "a b", "", 5]) {
            expect(legacyRedirect("/operators", { id })).toBeNull();
        }
    });
});
