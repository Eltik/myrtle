import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { ALL_ENTITY_KINDS } from "#/lib/api/tier-entities";
import { I18nProvider } from "#/lib/i18n/context";
import { kindDefinition, useEntityLabels, useKindT } from "./kinds";

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
        expect(ALL_ENTITY_KINDS).toEqual(["operator", "skill", "module", "skin", "class", "subclass", "faction", "enemy", "event", "integrated_strategies", "stronghold_bond", "story_sprite"]);
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
