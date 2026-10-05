import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { ALL_ENTITY_KINDS, isEntityOfKind, toTierEntity, UNPLACED } from "#/lib/api/tier-entities";
import { I18nProvider } from "#/lib/i18n/context";
import { compactForSearch } from "#/lib/search/fuzzy";
import { type KindT, kindDefinition, useEntityLabels, useKindT } from "./kinds";

afterEach(cleanup);

function KindLabels() {
    const labels = useEntityLabels();
    const t = useKindT();
    return (
        <ul>
            {ALL_ENTITY_KINDS.map((kind) => (
                <li key={kind} data-kind={kind}>
                    {[labels.plural(kind), labels.singular(kind), labels.description(kind), kindDefinition(kind).pool(t, []).kicker].join("|")}
                </li>
            ))}
        </ul>
    );
}

describe("KIND_DEFINITIONS", () => {
    it("lists every kind in the settings and tab order", () => {
        expect(ALL_ENTITY_KINDS).toEqual(["operator", "skill", "module", "skin", "class", "subclass", "faction", "enemy", "event", "main_story", "integrated_strategies", "stronghold_bond", "story_sprite"]);
    });

    it("gives every kind its own translated names, description and pool kicker, never a raw key", () => {
        const { container } = render(
            <I18nProvider locale="en" available={[{ code: "en", nativeName: "English" }]} messages={{}}>
                <KindLabels />
            </I18nProvider>,
        );
        for (const kind of ALL_ENTITY_KINDS) {
            const parts = container.querySelector(`[data-kind='${kind}']`)?.textContent?.split("|") ?? [];
            expect(parts).toHaveLength(4);
            for (const part of parts) {
                expect(part.length).toBeGreaterThan(0);
                expect(part).not.toMatch(/^(entity|edit)\./);
            }
        }
        expect(container.querySelector("[data-kind='enemy']")?.textContent).toBe("Enemies|Enemy|Every enemy in the handbook|Enemy pool");
    });
});

describe("story_sprite search", () => {
    // How the pool and the grid picker match: the compacted query inside a compacted search text.
    const finds = (query: string, aliases?: string[]) => {
        const facets = aliases ? { source: "npc", aliases } : { source: "npc" };
        const entity = toTierEntity("story_sprite", "avg_npc_061", { kind: "story_sprite", id: "avg_npc_061", name: "Maria", icon: null, href: null, facets }, UNPLACED);
        if (!isEntityOfKind(entity, "story_sprite")) throw new Error("unresolved");
        const q = compactForSearch(query);
        return kindDefinition("story_sprite")
            .pool(((key: string) => key) as unknown as KindT, [])
            .searchTexts(entity)
            .some((text) => Boolean(text) && compactForSearch(text ?? "").includes(q));
    };

    it("finds a character by a speaker name it is also spoken under", () => {
        expect(finds("Blemishine", ["Blemishine"])).toBe(true);
        expect(finds("blemi", ["Blemishine"])).toBe(true);
        expect(finds("Blemishine")).toBe(false);
        expect(finds("Maria")).toBe(true);
    });

    it("does not find a name no search text holds", () => {
        // No source string says "Nearl", so the full name finds nothing.
        expect(finds("Maria Nearl", ["Blemishine"])).toBe(false);
    });
});
