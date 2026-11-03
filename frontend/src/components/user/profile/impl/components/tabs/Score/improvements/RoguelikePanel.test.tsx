import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { IImprovementsResponse, IRoguelikeThemeImprovement } from "#/lib/api/user";
import { I18nProvider } from "#/lib/i18n/context";
import { endingsNote, RoguelikePanel } from "./RoguelikePanel";
import { messages } from "./RoguelikePanel.messages";

const MESSAGES = Object.fromEntries(Object.entries(messages).map(([key, m]) => [`user.${key}`, m.text]));

const pair = (current: number, max: number) => ({ current, max });

/** `rogue_3` as the EN game data has it: 5 listed endings, one from Deep Investigation. */
function theme(overrides: Partial<IRoguelikeThemeImprovement> = {}): IRoguelikeThemeImprovement {
    return {
        theme_id: "rogue_3",
        theme_name: "Expeditioner's Jǫklumarkar",
        endings: pair(2, 5),
        difficulty: { highest_cleared: 3, max: 15 },
        collectibles: { relics: pair(1, 10), capsules: pair(0, 0), bands: pair(1, 4) },
        bp: pair(4, 100),
        challenges: pair(0, 13),
        special_endings: [{ id: "ro3_ending_c", name: "The Fleetest of Light", mode_name: "Deep Investigation" }],
        ...overrides,
    };
}

function mount(themes: IRoguelikeThemeImprovement[]) {
    return render(
        <I18nProvider locale="en" available={[{ code: "en", nativeName: "English" }]} messages={MESSAGES}>
            <RoguelikePanel improvements={{ roguelike: themes } as IImprovementsResponse} accent="red" />
        </I18nProvider>,
    );
}

const LABEL = messages["score.improvements.roguelike.endings.special.label"].text;

afterEach(cleanup);

describe("endingsNote", () => {
    it("is null when the field is missing, which is every theme before the backend sent it", () => {
        const { special_endings: _, ...older } = theme();
        expect(endingsNote(older)).toBeNull();
    });

    it("is null for an empty list", () => {
        expect(endingsNote(theme({ special_endings: [] }))).toBeNull();
    });

    it("splits the shown count into regular and special without changing it", () => {
        expect(endingsNote(theme())).toEqual({ endings: theme().special_endings, regular: 4, total: 5 });
    });
});

describe("RoguelikePanel endings note", () => {
    it("puts an (i) beside the count that opens the explanation", async () => {
        mount([theme()]);
        // The count itself is unchanged: 2 / 5.
        expect(screen.getByText("/ 5")).toBeTruthy();
        const button = screen.getByRole("button", { name: LABEL });
        // A real button: reachable by Tab, opened by Enter/Space and by a tap.
        expect(button.tagName).toBe("BUTTON");
        fireEvent.click(button);
        expect(await screen.findByText("The Fleetest of Light comes from Deep Investigation. The game data lists it as an ending, so this counts 5: 4 from regular runs plus 1.")).toBeTruthy();
    });

    it("words a special ending with no mode name without inventing one", async () => {
        mount([theme({ special_endings: [{ id: "x", name: "Hidden Path" }] })]);
        fireEvent.click(screen.getByRole("button", { name: LABEL }));
        expect(await screen.findByText(/^Hidden Path is not reached in a regular run\. /)).toBeTruthy();
    });

    it("renders no (i) for a theme without special endings, missing or empty", () => {
        const { special_endings: _, ...older } = theme({ theme_id: "rogue_4", theme_name: "Sarkaz's Furnaceside Fables" });
        mount([older, theme({ theme_id: "rogue_5", special_endings: [] })]);
        expect(screen.queryByRole("button", { name: LABEL })).toBeNull();
    });
});
