import { describe, expect, it } from "vitest";
import { spriteThumbUrl } from "./thumb";

describe("an expression's thumb URL", () => {
    it("encodes the key and carries the server only off the default", () => {
        expect(spriteThumbUrl("avg_npc_043_1", "#2$1", "en")).toMatch(/\/api\/story\/sprites\/avg_npc_043_1\/thumb\/%232%241$/);
        expect(spriteThumbUrl("avg_npc_043_1", "@smile", "cn")).toMatch(/\/api\/cn\/story\/sprites\/avg_npc_043_1\/thumb\/%40smile$/);
    });
});
