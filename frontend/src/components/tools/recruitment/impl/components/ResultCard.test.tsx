import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { I18nProvider } from "#/lib/i18n/context";
import type { IRecruitableOperator, IRosterOverlay, ITagCombinationResult } from "../types";
import { ResultCard } from "./ResultCard";
import { messages } from "./ResultCard.messages";

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

function mount(roster: IRosterOverlay | null) {
    return render(
        <I18nProvider locale="en" available={[{ code: "en", nativeName: "English" }]} messages={MESSAGES}>
            <ResultCard result={RESULT} roster={roster} />
        </I18nProvider>,
    );
}

afterEach(cleanup);

describe("ResultCard roster overlay", () => {
    it("labels each row with what the next potential grants", () => {
        mount(ROSTER);
        const rows = screen.getAllByRole("listitem").map((li) => li.textContent ?? "");
        expect(rows.map((r) => r.replace(/^.*·Guard·/, ""))).toEqual(["Not owned", "DP cost -1", "Talent 2", "ATK +28", "Talent", "Redeploy -4s", "Maxed"]);
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
