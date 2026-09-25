import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { I18nProvider } from "#/lib/i18n/context";
import type { IRecruitableOperator, IRosterOverlay, ITagCombinationResult } from "../types";
import { ResultCard } from "./ResultCard";
import { messages } from "./ResultCard.messages";
import { ResultCardDetailed } from "./ResultCardDetailed";

const MESSAGES = Object.fromEntries(Object.entries(messages).map(([key, m]) => [`tools.${key}`, m.text]));

function op(id: string, potentials: IRecruitableOperator["potentials"]): IRecruitableOperator {
    return { id, name: id, rarity: 4, profession: "WARRIOR", position: "MELEE", tagList: [], potentials };
}

const FIVE = [
    { kind: "stat" as const, attribute: "COST", value: -1 },
    { kind: "talent" as const, index: 1, of: 2 },
    { kind: "stat" as const, attribute: "ATK", value: 28 },
    { kind: "talent" as const, index: 0, of: 1 },
    { kind: "stat" as const, attribute: "RESPAWN_TIME", value: -4 },
];

const RESULT: ITagCombinationResult = {
    tags: [9],
    tagNames: ["Melee"],
    operators: [op("unowned", FIVE), op("p1", FIVE), op("p2", FIVE), op("p3", FIVE), op("p4", FIVE), op("p5", FIVE), op("p6", FIVE)],
    guaranteedRarity: 4,
    maxRarity: 4,
    fiveStarCount: 0,
};

const ROSTER: IRosterOverlay = {
    potentialByOperator: new Map([
        ["p1", 0],
        ["p2", 1],
        ["p3", 2],
        ["p4", 3],
        ["p5", 4],
        ["p6", 5],
    ]),
    showPotentials: true,
    showNextUpgrade: true,
};

function mount(roster: IRosterOverlay | null, Card: typeof ResultCard = ResultCard) {
    return render(
        <I18nProvider locale="en" available={[{ code: "en", nativeName: "English" }]} messages={MESSAGES}>
            <Card result={RESULT} roster={roster} />
        </I18nProvider>,
    );
}

afterEach(cleanup);

describe("ResultCard roster overlay", () => {
    it("labels each tile with what the next potential grants", () => {
        const { container } = mount(ROSTER);
        const tiles = screen.getAllByRole("listitem");
        expect(tiles).toHaveLength(7);
        const labels = [...container.querySelectorAll('[data-slot="recruit-next-potential"]')].map((el) => el.textContent);
        expect(labels).toEqual(["Not owned", "DP cost -1", "Talent 2", "ATK +28", "Talent", "Redeploy -4s", "Maxed"]);
    });

    it("hides the next-potential line when the option is off", () => {
        const { container } = mount({ ...ROSTER, showNextUpgrade: false });
        expect(container.querySelectorAll('[data-slot="recruit-next-potential"]')).toHaveLength(0);
        expect(screen.getAllByRole("img", { name: /^Potential \d$/ })).toHaveLength(5);
    });

    it("overlays the potential icon only on owned operators above P1", () => {
        mount(ROSTER);
        const icons = screen.getAllByRole("img", { name: /^Potential \d$/ }).map((img) => img.getAttribute("alt"));
        expect(icons).toEqual(["Potential 2", "Potential 3", "Potential 4", "Potential 5", "Potential 6"]);
    });

    it("renders no roster text at all without an overlay", () => {
        mount(null);
        const text = screen
            .getAllByRole("listitem")
            .map((li) => li.textContent ?? "")
            .join("|");
        expect(text).not.toMatch(/Not owned|Maxed|DP cost|Talent/);
        expect(screen.queryAllByRole("img", { name: /^Potential/ })).toHaveLength(0);
    });
});

describe("ResultCardDetailed roster overlay", () => {
    const mountDetailed = (roster: IRosterOverlay | null) => mount(roster, ResultCardDetailed);

    it("labels each row with what the next potential grants", () => {
        mountDetailed(ROSTER);
        const rows = screen.getAllByRole("listitem").map((li) => li.textContent ?? "");
        expect(rows.map((r) => r.replace(/^.*·Guard·/, ""))).toEqual(["Not owned", "DP cost -1", "Talent 2", "ATK +28", "Talent", "Redeploy -4s", "Maxed"]);
    });

    it("hides the next-potential text when the option is off", () => {
        const { container } = mountDetailed({ ...ROSTER, showNextUpgrade: false });
        expect(container.querySelectorAll('[data-slot="recruit-next-potential"]')).toHaveLength(0);
        expect(screen.getAllByRole("img", { name: /^Potential \d$/ })).toHaveLength(5);
    });

    it("overlays the potential icon only on owned operators above P1", () => {
        mountDetailed(ROSTER);
        const icons = screen.getAllByRole("img", { name: /^Potential \d$/ }).map((img) => img.getAttribute("alt"));
        expect(icons).toEqual(["Potential 2", "Potential 3", "Potential 4", "Potential 5", "Potential 6"]);
    });

    it("renders no roster text at all without an overlay", () => {
        mountDetailed(null);
        const text = screen
            .getAllByRole("listitem")
            .map((li) => li.textContent ?? "")
            .join("|");
        expect(text).not.toMatch(/Not owned|Maxed|DP cost|Talent/);
        expect(screen.queryAllByRole("img", { name: /^Potential/ })).toHaveLength(0);
    });
});

describe.each([
    ["compact", ResultCard],
    ["detailed", ResultCardDetailed],
] as const)("%s layout unowned greying", (_name, Card) => {
    it("greys only the unowned portrait while potentials are shown", () => {
        const { container } = mount(ROSTER, Card);
        expect(container.querySelectorAll(".grayscale")).toHaveLength(1);
    });

    it("greys nothing when potentials are hidden", () => {
        const { container } = mount({ ...ROSTER, showPotentials: false }, Card);
        expect(container.querySelectorAll(".grayscale")).toHaveLength(0);
    });
});
