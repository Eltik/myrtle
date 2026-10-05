import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, describe, expect, it } from "vitest";
import { env } from "#/env";
import { entityArtFit, entityOwner, type ITierEntity, integratedStrategiesNumber, toTierEntity } from "#/lib/api/tier-entities";
import { I18nProvider } from "#/lib/i18n/context";
import type { EntitySummary } from "#/types/generated/EntitySummary";
import { EntityTile } from "./detail/EntityTile";
import { DragControllerProvider } from "./edit/drag-controller";
import { EditableOpTile } from "./edit/EditableOpTile";
import { KindPool, PAGE } from "./edit/KindPool";
import { PickTierDialog } from "./edit/PickTierDialog";
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

const theme = entity({ kind: "integrated_strategies", id: "rogue_1", name: "Phantom & Crimson Solitaire", icon: "/assets/textures/spritepack/ui_zone_home_theme_rogue_1_entry_display_1/rogue_1_entry_display_1.png", facets: { theme: "rogue_1", item_type: "theme" } });
const relic = entity({ kind: "integrated_strategies", id: "rogue_1_relic_a01", name: "Relic", icon: "/assets/textures/spritepack/ui_roguelike_topic_item_h1_rogue_1_0/rogue_1_relic_a01.png", facets: { theme: "rogue_1", item_type: "relic" } });
const moduleArt = entity({ kind: "module", id: "uniequip_002_hsguma", name: "Stick and Sack", icon: "/assets/textures/spritepack/ui_equip_big_img_hub_0/uniequip_002_hsguma.png", facets: { char_id: "char_136_hsguma", operator: "Hoshiguma", module_type: "X", type_code: "PRO" } });
const moduleBadge = entity({ kind: "module", id: "uniequip_002_x", name: "Badge only", icon: "/assets/textures/spritepack/ui_equip_type_direction_hub_h2_0/pro-x.png", facets: { char_id: "char_x", module_type: "X" } });
const sprite = entity({ kind: "story_sprite", id: "avg_npc_005", name: "Old Man", icon: "/story-sprite-thumb/avg_npc_005", facets: { source: "npc" } });

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

    it("reads the newer kinds' facets", () => {
        expect(entity({ kind: "skill", id: "char_002_amiya:skchr_amiya_2", name: "Spirit Burst", icon: null, facets: { char_id: "char_002_amiya", operator: "Amiya", slot: "2", skill_type: "manual", sp_type: "auto", profession: "CASTER" } })).toMatchObject({
            kind: "skill",
            charId: "char_002_amiya",
            slot: 2,
            spType: "auto",
        });
        expect(entity({ kind: "skill", id: "char_x:skchr_x_1", name: "Passive", icon: null, facets: { slot: "1", skill_type: "passive" } })).toMatchObject({ spType: null, operatorName: null });
        expect(entity({ kind: "skin", id: "char_002_amiya@winter#1", name: "Winter Messenger", icon: null, facets: { char_id: "char_002_amiya", rarity: "5", brand: "Icefield Messenger" } })).toMatchObject({ kind: "skin", rarity: 5, brand: "Icefield Messenger" });
        expect(entity({ kind: "module", id: "uniequip_002_amiya", name: "Module", icon: null, facets: { module_type: "D", type_code: "CCR" } })).toMatchObject({ kind: "module", moduleType: "D", typeCode: "CCR" });
        expect(entity({ kind: "integrated_strategies", id: "rogue_2", name: "Mizuki & Caerula Arbor", icon: null, facets: { theme: "rogue_2", item_type: "theme" } })).toMatchObject({ theme: "rogue_2", itemType: "theme" });
        expect(entity({ kind: "story_sprite", id: "avg_npc_935", name: "Ines", icon: null, facets: { source: "npc" } })).toMatchObject({ source: "npc" });
        expect(entity({ kind: "main_story", id: "main_14", name: "Absolved Will Be the Seekers", icon: null, facets: { episode: "14", act: "2", act_name: "Shadow of A Dying Sun" } })).toMatchObject({ episode: 14, act: 2, actName: "Shadow of A Dying Sun" });
        expect(entity({ kind: "main_story", id: "main_99", name: "Unnumbered", icon: null, facets: {} })).toMatchObject({ episode: null, act: null, actName: null });
    });

    it("numbers Integrated Strategies themes the way the game does, one past the table", () => {
        expect(integratedStrategiesNumber("rogue_1")).toBe(2);
        expect(integratedStrategiesNumber("rogue_2")).toBe(3);
        expect(integratedStrategiesNumber("nonsense")).toBeNull();
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

    it("fills the tile with a picture and insets only icon-style art, per the kind registry", () => {
        expect([theme, moduleArt, sprite, enemy, operator].map(entityArtFit)).toEqual(["cover", "cover", "cover", "cover", "cover"]);
        expect([relic, moduleBadge].map(entityArtFit)).toEqual(["object", "object"]);
        expect([faction, bond].map(entityArtFit)).toEqual(["glyph", "glyph"]);
        const { container } = render(wrap(<EntityAvatar entity={theme} face="tile" />));
        const img = container.querySelector("img");
        expect(img?.className).toContain("object-cover");
        expect(img?.className).not.toContain("p-[");
    });

    it("shows a story sprite's backend thumbnail as a plain covering image, no CSS crop", () => {
        const { container } = render(wrap(<EntityAvatar entity={sprite} face="tile" />));
        const img = container.querySelector("img");
        expect(img?.getAttribute("src")).toBe(`${BASE}/api/story-sprite-thumb/avg_npc_005`);
        expect(img?.className).toContain("object-cover");
        expect(img?.getAttribute("style")).toBeNull();
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

    it("names a module's operator in its accessible name and tooltip", () => {
        expect(entityOwner(moduleArt)).toBe("Hoshiguma");
        expect([theme, enemy, operator].map(entityOwner)).toEqual([null, null, null]);
        render(wrap(<EditableOpTile entity={moduleArt} />));
        const tile = screen.getByRole("button");
        expect(tile.getAttribute("aria-label")).toBe("Stick and Sack (Hoshiguma), Module");
        expect(tile.getAttribute("title")).toBe("Stick and Sack (Hoshiguma)");
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

describe("PickTierDialog", () => {
    it("says the list has no tiers instead of pointing at an empty list", () => {
        render(wrap(<PickTierDialog entity={event} currentTierId={null} description="" tiers={[]} onClose={() => {}} onPick={() => {}} onDescriptionChange={() => {}} />));
        expect(screen.getByText("No tiers yet. Close this dialog and add a tier to the board first.")).toBeTruthy();
        expect(screen.queryByText("Pick a tier above to save this note with the placement.")).toBeNull();
        expect(document.body.textContent).not.toMatch(/operator/i);
    });
});

describe("PickTierDialog owner", () => {
    it("puts the owning operator under a module's name", () => {
        render(wrap(<PickTierDialog entity={moduleArt} currentTierId={null} description="" tiers={[]} onClose={() => {}} onPick={() => {}} onDescriptionChange={() => {}} />));
        expect(screen.getByText("Stick and Sack")).toBeTruthy();
        expect(screen.getByText("Operator: Hoshiguma")).toBeTruthy();
    });
});

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

    it("filters Integrated Strategies by theme, labelled with the game's IS number", () => {
        const entries: ITierEntity[] = [
            entity({ kind: "integrated_strategies", id: "rogue_1", name: "Phantom & Crimson Solitaire", icon: null, facets: { theme: "rogue_1", item_type: "theme" } }),
            entity({ kind: "integrated_strategies", id: "rogue_2_relic_1", name: "Relic", icon: null, facets: { theme: "rogue_2", item_type: "relic" } }),
        ];
        render(wrap(<KindPool kind="integrated_strategies" entities={entries} status="success" placedKeys={new Set()} onUnplace={() => {}} onPickerActivate={() => {}} />));
        fireEvent.click(screen.getByRole("button", { name: /expand/i }));
        expect(screen.getByRole("button", { name: "IS2" })).toBeTruthy();
        expect(screen.getByRole("button", { name: "IS3" })).toBeTruthy();
        expect(screen.getByRole("button", { name: "Collectible" })).toBeTruthy();
    });

    it("shows a loading state, not an empty one, while a catalogue loads", () => {
        render(wrap(<KindPool kind="event" entities={undefined} status="pending" placedKeys={new Set()} onUnplace={() => {}} onPickerActivate={() => {}} />));
        expect(screen.getByText("Loading…")).toBeTruthy();
        expect(screen.queryByText("No matches")).toBeNull();
    });
});
