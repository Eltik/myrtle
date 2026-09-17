import { describe, expect, it } from "vitest";
import { scoreMatch, searchAndRank } from "./fuzzy";

const aglna2 = { name: "予愿安洁莉娜", aliases: ["Angelina the Mellow Wish"], extra: "Supporter 6★" };
const angelina = { name: "Angelina", extra: "Angelina Supporter 6★" };

describe("scoreMatch aliases", () => {
    it("scores an alias on the name tiers, not the extra tier", () => {
        expect(scoreMatch("angelina", aglna2)).toBe(600 + 0); // prefix of a 24-char alias: no length bonus
        expect(scoreMatch("mellow", aglna2)).toBe(450 - 13); // word prefix at index 13
        expect(scoreMatch("angelina the mellow wish", aglna2)).toBe(1000);
    });

    it("keeps the exact name ahead of an alias prefix", () => {
        const ranked = searchAndRank("angelina", [aglna2, angelina], (t) => t);
        expect(ranked.map((r) => r.item.name)).toEqual(["Angelina", "予愿安洁莉娜"]);
    });

    it("is unchanged for targets without aliases", () => {
        expect(scoreMatch("angelina", angelina)).toBe(1000 + 12);
        expect(scoreMatch("supporter", angelina)).toBe(90);
        expect(scoreMatch("zzz", angelina)).toBe(0);
    });

    it("ignores blank aliases", () => {
        expect(scoreMatch("lancet", { name: "Lancet-2", aliases: [" "] })).toBe(600 + 13); // "lancet2" after punctuation strip: 7 chars
    });
});
