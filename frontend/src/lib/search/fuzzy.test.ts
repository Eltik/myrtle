import { describe, expect, it } from "vitest";
import { prepareQuery, prepareTarget, scoreMatch, scorePrepared, searchAndRank } from "./fuzzy";

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

describe("scorePrepared", () => {
    const targets = [
        { name: "予愿安洁莉娜", aliases: ["Angelina the Mellow Wish", " "], extra: "Supporter 6★" },
        { name: "Angelina", extra: "Angelina Supporter 6★" },
        { name: "Lancet-2" },
        { name: "Miss.Christine", extra: "Specialist  Dollkeeper" },
        { name: "Łódź Ænigma", extra: "Crème brûlée" },
        { name: "Under Tides", extra: "Beach Episode DH-1 Ancient Forge" },
    ];
    const queries = ["", "  ", "a", "angelina", "mellow", "lancet", "lancet2", "miss christine", "misschristine", "lodz", "aenigma", "creme", "beach ep", "dh1", "dh-1", "utd", "zzz", "ANGELINA"];

    // The scores the scorer gave BEFORE the prepared path existed (HEAD
    // deb2eb5f), one row per target, one column per query. Both entry points
    // must still give exactly these.
    const before = [
        [1, 1, 600, 600, 437, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 600],
        [1, 1, 612, 1012, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1012],
        [1, 1, 312, 0, 0, 613, 1013, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [1, 1, 90, 0, 0, 0, 0, 307, 1007, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [1, 1, 453, 0, 0, 0, 0, 0, 0, 608, 453, 90, 0, 0, 0, 0, 0, 0],
        [1, 1, 90, 0, 0, 0, 0, 0, 0, 0, 0, 0, 90, 90, 90, 189, 0, 0],
    ];

    it("gives the pre-change score for every query and target, on both entry points", () => {
        targets.forEach((target, row) => {
            const prepared = prepareTarget(target);
            queries.forEach((query, col) => {
                const want = before[row]?.[col];
                expect(scoreMatch(query, target), `${query} / ${target.name}`).toBe(want);
                expect(scorePrepared(prepareQuery(query), prepared), `${query} / ${target.name}`).toBe(want);
            });
        });
    });

    it("treats a blank query as the small positive score", () => {
        expect(prepareQuery("   ")).toBeNull();
        expect(scorePrepared(null, prepareTarget(targets[0] ?? { name: "" }))).toBe(1);
    });
});
