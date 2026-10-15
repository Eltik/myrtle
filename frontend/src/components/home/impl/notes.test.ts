import { describe, expect, it } from "vitest";
import { splitNoteTitle } from "./notes";

describe("splitNoteTitle", () => {
    it("lifts a short kind prefix off the title", () => {
        expect(splitNoteTitle("New: Make your profile your own")).toEqual({ kind: "New", title: "Make your profile your own" });
        expect(splitNoteTitle("改进：页面加载更快")).toEqual({ kind: "改进", title: "页面加载更快" });
    });

    it("leaves a title with no prefix whole", () => {
        expect(splitNoteTitle("Grids are here")).toEqual({ kind: null, title: "Grids are here" });
    });
});
