import { describe, expect, it } from "vitest";
import type { ITierListBrowseItem } from "#/lib/api/tier-lists";
import { authorsOf } from "./AuthorFilter";

function list(authorId: string | null, name: string): ITierListBrowseItem {
    return { author: { id: authorId, name, avatarId: null } } as ITierListBrowseItem;
}

describe("authorsOf", () => {
    it("counts each author's lists, most first, and skips lists without an author", () => {
        const authors = authorsOf([list("a", "Amiya"), list("b", "Blaze"), list("b", "Blaze"), list(null, "Community")]);
        expect(authors.map((a) => [a.id, a.lists])).toEqual([
            ["b", 2],
            ["a", 1],
        ]);
    });
});
