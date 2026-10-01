import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, describe, expect, it } from "vitest";
import { env } from "#/env";
import { type ITierEntity, toTierEntity } from "#/lib/api/tier-entities";
import { I18nProvider } from "#/lib/i18n/context";
import type { EntitySummary } from "#/types/generated/EntitySummary";
import { EntityTile } from "./detail/EntityTile";
import { DragControllerProvider } from "./edit/drag-controller";
import { EditableOpTile } from "./edit/EditableOpTile";
import { KindPool, PAGE } from "./edit/KindPool";
import { EntityAvatar, entityInitials, entityShape } from "./entities";

// The tiles and the pool are user-facing and the backend cannot run here, so
// these mount them in jsdom and assert on the DOM a browser would get. What
// they cannot show is pixels: sizes, crops and the light/dark inversion are
// CSS, checked by reading the rules, not by rendering them.

/** What every icon URL starts with on this deployment. */
const BASE = env.VITE_BACKEND_URL ?? "";

const PLACED = { subOrder: 0, description: null, updatedAt: "2026-10-01T00:00:00Z" };

function entity(summary: Omit<EntitySummary, "href"> & { href?: string | null }): ITierEntity {
    return toTierEntity(summary.kind, summary.id, { href: null, ...summary }, PLACED);
}

const faction = entity({ kind: "faction", id: "rhodes", name: "Rhodes Island", icon: "/assets/textures/spritepack/ui_camp_logo_0/logo_rhodes.png", facets: { power_level: "nation" } });
const enemy = entity({ kind: "enemy", id: "enemy_1007_slime", name: "Originium Slug", icon: "/enemy-icon/enemy_1007_slime", href: "/enemies/enemy_1007_slime", facets: { enemy_level: "ELITE", enemy_index: "B1" } });
const event = entity({ kind: "event", id: "act24side", name: "Ideal City", icon: null, facets: { display_type: "SIDESTORY", start_time: "1690000000", type: "TYPE_ACT24SIDE" } });
const bond = entity({ kind: "stronghold_bond", id: "yanShip", name: "Yan", icon: "/assets/textures/ui/autochess/%5Buc%5Dautochesscommon/icon_yanShip.png", facets: { bond_type: "season", power_ids: ["yan", "sui"] } });
const operator = entity({ kind: "operator", id: "char_002_amiya", name: "Amiya", icon: "/avatar/char_002_amiya", facets: { rarity: "5", profession: "CASTER", sub_profession_id: "corecaster", position: "RANGED" } });

function wrap(children: ReactNode) {
    return (
        <I18nProvider locale="en" available={[{ code: "en", nativeName: "English" }]} messages={{}}>
            <DragControllerProvider entityByKey={{}} onPlace={() => {}} onUnplace={() => {}}>
                {children}
            </DragControllerProvider>
        </I18nProvider>
    );
}

afterEach(cleanup);

describe("toTierEntity", () => {
    it("reads each kind's facets into its own fields", () => {
        expect(faction).toMatchObject({ kind: "faction", resolved: true, powerLevel: "nation" });
        expect(enemy).toMatchObject({ kind: "enemy", resolved: true, level: "ELITE", index: "B1" });
        expect(event).toMatchObject({ kind: "event", resolved: true, displayType: "SIDESTORY", startTime: 1690000000, rerun: false });
        expect(bond).toMatchObject({ kind: "stronghold_bond", resolved: true, bondType: "season" });
        expect(entity({ kind: "subclass", id: "charger", name: "Charger", icon: null, facets: { profession: "PIONEER" } })).toMatchObject({ kind: "subclass", profession: "PIONEER" });
        expect(entity({ kind: "event", id: "act9sre", name: "Who Is Real - Rerun", icon: null, facets: { display_type: "SIDESTORY", start_time: "1", rerun: "true" } })).toMatchObject({ rerun: true });
    });

    it("only an event is wide", () => {
        expect(entityShape(event)).toBe("wide");
        expect([faction, enemy, bond, operator].map(entityShape)).toEqual(["square", "square", "square", "square"]);
    });
});

describe("entityInitials", () => {
    it("takes two words' initials, or a lone word's first two letters", () => {
        expect(entityInitials("Rhodes Island")).toBe("RI");
        expect(entityInitials("Yan")).toBe("YA");
        expect(entityInitials("S.W.E.E.P.")).toBe("SW");
        expect(entityInitials("")).toBe("?");
    });
});

describe("EntityAvatar", () => {
    it("draws a glyph kind inset and theme-inverted, from its API path", () => {
        const { container } = render(wrap(<EntityAvatar entity={faction} />));
        const img = container.querySelector("img");
        expect(img?.getAttribute("src")).toBe(`${BASE}/api/assets/textures/spritepack/ui_camp_logo_0/logo_rhodes.png`);
        expect(img?.className).toContain("object-contain");
        expect(img?.className).toContain("icon-theme-aware");
    });

    it("never inverts a glyph on the always-dark drag ghost", () => {
        const { container } = render(wrap(<EntityAvatar entity={bond} tone="dark" />));
        expect(container.querySelector("img")?.className).not.toContain("icon-theme-aware");
    });

    it("fills the tile with an enemy portrait", () => {
        const { container } = render(wrap(<EntityAvatar entity={enemy} />));
        const img = container.querySelector("img");
        expect(img?.getAttribute("src")).toBe(`${BASE}/api/enemy-icon/enemy_1007_slime`);
        expect(img?.className).toContain("object-cover");
    });

    it("falls back to initials when the image fails, never a broken image", () => {
        const { container } = render(wrap(<EntityAvatar entity={faction} />));
        const img = container.querySelector("img");
        if (!img) throw new Error("expected an img");
        fireEvent.error(img);
        expect(container.querySelector("img")).toBeNull();
        expect(container.textContent).toBe("RI");
    });

    it("spells an art-less event's name across its wide tile, initials in a chip", () => {
        const tile = render(wrap(<EntityAvatar entity={event} face="tile" />));
        expect(tile.container.textContent).toBe("Ideal City");
        cleanup();
        const chip = render(wrap(<EntityAvatar entity={event} />));
        expect(chip.container.textContent).toBe("IC");
    });

    it("reads the reader's game server when it is not the default", () => {
        const { container } = render(
            <I18nProvider locale="en" available={[{ code: "en", nativeName: "English" }]} messages={{}} gamedataServer="cn">
                <EntityAvatar entity={enemy} />
            </I18nProvider>,
        );
        expect(container.querySelector("img")?.getAttribute("src")).toBe(`${BASE}/api/cn/enemy-icon/enemy_1007_slime`);
    });

    it("leaves an operator's avatar exactly as it was", () => {
        const { container } = render(wrap(<EntityAvatar entity={operator} />));
        expect(container.querySelector("img")?.getAttribute("src")).toBe(`${BASE}/api/avatar/char_002_amiya`);
    });
});

describe("EditableOpTile", () => {
    it("marks a non-operator with its kind and shape, and names it with its kind", () => {
        render(wrap(<EditableOpTile entity={event} />));
        const tile = screen.getByRole("button");
        expect(tile.getAttribute("data-kind")).toBe("event");
        expect(tile.getAttribute("data-shape")).toBe("wide");
        expect(tile.getAttribute("aria-label")).toBe("Ideal City, Event");
    });

    it("gives an operator no kind attributes, so its tile is the one it always was", () => {
        render(wrap(<EditableOpTile entity={operator} />));
        const tile = screen.getByRole("button");
        expect(tile.hasAttribute("data-kind")).toBe(false);
        expect(tile.hasAttribute("data-shape")).toBe(false);
        expect(tile.getAttribute("aria-label")).toBe("Amiya (5★)");
    });
});

describe("EntityTile", () => {
    it("makes a kind with no page a focusable tile named with its kind", () => {
        const { container } = render(wrap(<EntityTileNoRouter entity={faction} />));
        const tile = container.querySelector("[data-kind='faction']");
        expect(tile?.getAttribute("aria-label")).toBe("Rhodes Island, Faction");
        expect(tile?.getAttribute("tabindex")).toBe("0");
    });
});

/** `EntityTile` for a kind with no page renders no router Link, so it mounts without a router. */
function EntityTileNoRouter({ entity: e }: { entity: ITierEntity }) {
    return <EntityTile entity={e} />;
}

describe("KindPool", () => {
    const enemies: ITierEntity[] = Array.from({ length: 1541 }, (_, i) => entity({ kind: "enemy", id: `enemy_${i}`, name: `Enemy ${i}`, icon: `/enemy-icon/enemy_${i}`, facets: { enemy_level: i % 10 === 0 ? "BOSS" : "NORMAL", enemy_index: `E${i}` } }));
    const operators: ITierEntity[] = Array.from({ length: 409 }, (_, i) => entity({ kind: "operator", id: `char_${i}`, name: `Op ${i}`, icon: `/avatar/char_${i}`, facets: { rarity: String((i % 6) + 1), profession: "CASTER", sub_profession_id: "x", position: "RANGED" } }));

    function pool(kind: "enemy" | "operator", entities: ITierEntity[]) {
        return render(wrap(<KindPool kind={kind} entities={entities} status="success" placedKeys={new Set()} onUnplace={() => {}} onPickerActivate={() => {}} />));
    }

    it("renders a 1,541-enemy catalogue one page at a time", () => {
        const { container } = pool("enemy", enemies);
        expect(container.querySelectorAll("[data-kind='enemy']").length).toBe(PAGE);
        expect(container.textContent).toContain(`Showing ${PAGE} of 1541. Scroll for more.`);
        expect(screen.getByText("Enemy pool")).toBeTruthy();
    });

    it("renders the 409-operator catalogue whole, as before", () => {
        const { container } = pool("operator", operators);
        expect(container.querySelectorAll("button[data-tl-chip-id]").length).toBe(409);
        expect(container.textContent).not.toContain("Scroll for more");
        expect(screen.getByText("Operator pool")).toBeTruthy();
    });

    it("searches an enemy by its handbook code", () => {
        const { container } = pool("enemy", enemies);
        const search = screen.getByRole("searchbox", { name: "Search enemies" });
        fireEvent.change(search, { target: { value: "E1540" } });
        const tiles = container.querySelectorAll("[data-kind='enemy']");
        expect(tiles.length).toBe(1);
        expect(tiles[0]?.getAttribute("aria-label")).toBe("Enemy 1540, Enemy");
    });

    it("shows a loading state, not an empty one, while a catalogue loads", () => {
        render(wrap(<KindPool kind="event" entities={undefined} status="pending" placedKeys={new Set()} onUnplace={() => {}} onPickerActivate={() => {}} />));
        expect(screen.getByText("Loading…")).toBeTruthy();
        expect(screen.queryByText("No matches")).toBeNull();
    });
});
