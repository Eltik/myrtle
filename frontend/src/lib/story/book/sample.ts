/**
 * A small two-group library with scripts, for the renderer tests: a chapter
 * whose first operation has a Before and an After, an interlude, and a second
 * group so a selection spans two parts. Not a test file itself.
 */
import type { BookIndex, BookSource } from "./book";
import { decide, gate, say, scriptOf } from "./fixtures";

const assets = {
    backgrounds: { bg_room: "/textures/avg/bg/room.png", bg_street: "/textures/avg/bg/street.png" },
    images: { cg_one: "/textures/avg/imgs/cg_one.png", cg_two: "/textures/avg/imgs/cg_two.png" },
};

export const sampleIndex: BookIndex = {
    groups: [
        {
            id: "main_0",
            name: "Evil Time Part 1",
            bannerUrl: "/kv.png",
            titleImageUrl: "/title.png",
            illustrationCount: 4,
            stories: [
                { id: "s_01_end", name: "Isolated Island", code: "0-1", sort: 2, avgTag: "After Operation", hasScript: true, wordCount: 30 },
                { id: "s_01_beg", name: "Isolated Island", code: "0-1", sort: 1, avgTag: "Before Operation", hasScript: true, wordCount: 40 },
                { id: "s_int", name: "Prologue <1>", sort: 3, avgTag: "Interlude", hasScript: true, wordCount: 10 },
                { id: "s_none", name: "No script", code: "0-2", sort: 4, hasScript: false, wordCount: 0 },
            ],
        },
        { id: "act_x", name: "Other & Co", coverUrl: "/cover.png", stories: [{ id: "x_1", name: "Elsewhere", code: "X-1", sort: 1, hasScript: true, wordCount: 5 }] },
    ],
};

export const sampleSource: BookSource = {
    server: "en",
    index: sampleIndex,
    scripts: new Map([
        [
            "s_01_beg",
            scriptOf(
                "s_01_beg",
                [
                    ["background", { image: "bg_room" }],
                    say("Amiya", "Doctor, can you hear me? <i>Please</i> wake up."),
                    ["image", { image: "cg_one" }],
                    say("{@nickname}", "...Where am I?"),
                    ["text", {}, "The room is <color=#ff6600>burning</color>."],
                    decide(["Run.", "Stay & fight <here>."], ["1", "2"]),
                    gate("1"),
                    say("Amiya", "This way!"),
                    gate("2"),
                    ["background", { image: "bg_street" }],
                    say("Amiya", "Then we fight."),
                    gate("1", "2"),
                    ["sticker", { id: "st", text: "Chernobog\\n11:47 A.M." }],
                ],
                assets,
                "Amiya wakes the Doctor.",
            ),
        ],
        ["s_01_end", scriptOf("s_01_end", [["background", { image: "bg_street" }], say("Ace", "Go, {@nickname}!"), ["image", { image: "cg_one" }], ["image", { image: "cg_two" }]], assets)],
        ["s_int", scriptOf("s_int", [["subtitle", { text: "Some time later." }], say("", "A voice in the dark.")], assets)],
        ["x_1", scriptOf("x_1", [say("W", "Boom.")], assets)],
    ]),
};
