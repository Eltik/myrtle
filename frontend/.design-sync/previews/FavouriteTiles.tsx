import { FavouriteTiles } from "frontend";
import type { IShowcaseEntity } from "../../src/components/user/profile/impl/showcase";
import type { ITierEntity } from "../../src/lib/api/tier-entities";

// FavouriteTiles is the body of a profile Showcase "favourites" block: the
// picked entities as one wrapping row of tiles. Operators stand as the
// operators page's 2:3 card (portrait, faint faction mark, name + class on a
// frosted bar, rarity line); every other kind keeps its art's own shape at one
// shared height — square for outfits and main-story posters, a 2.1x-wide
// banner for events — on a dark tile with a hairline in the kind's accent.
// An id the loaded game data no longer knows renders as a dashed placeholder
// (only the owner ever sees it; it is dropped on save). Entities are real EN
// catalogue rows (`/api/tier-lists/catalogue/<kind>`, operator ids checked
// against `/api/operators/index`).

const BASE = { subOrder: 0, description: null, updatedAt: "2024-05-12T09:00:00.000Z", resolved: true as const };

function op(id: string, name: string, rarity: number, profession: string, subProfessionId: string, subProfessionName: string, professionName: string, position: string, nationId: string | null, nationName: string | null): IShowcaseEntity {
    const entity = { ...BASE, key: `operator:${id}`, kind: "operator", id, name, icon: `/avatar/${id}`, href: `/operators/${id}`, facets: { rarity: String(rarity) }, appellation: null, rarity: `TIER_${rarity}`, profession, subProfessionId, professionName, subProfessionName, position, nationId, nationName };
    return { id, entity: entity as unknown as ITierEntity, server: null };
}

function skin(id: string, name: string, charId: string, operatorName: string, rarity: number, profession: string, brand: string): IShowcaseEntity {
    const entity = { ...BASE, key: `skin:${id}`, kind: "skin", id, name, icon: `/avatar/${id.replace("@", "_").replace("#", "%23")}`, href: `/operators/${charId}`, facets: {}, charId, operatorName, rarity: `TIER_${rarity}`, profession, brand };
    return { id, entity: entity as unknown as ITierEntity, server: null };
}

function event(id: string, name: string, displayType: string, startTime: number): IShowcaseEntity {
    const entity = { ...BASE, key: `event:${id}`, kind: "event", id, name, icon: `/event-image/${id}`, href: null, facets: {}, displayType, startTime, rerun: false };
    return { id, entity: entity as unknown as ITierEntity, server: null };
}

function mainStory(id: string, name: string, icon: string, episode: number, act: number, actName: string): IShowcaseEntity {
    const entity = { ...BASE, key: `main_story:${id}`, kind: "main_story", id, name, icon, href: null, facets: {}, episode, act, actName };
    return { id, entity: entity as unknown as ITierEntity, server: null };
}

const OPERATORS = [
    op("char_003_kalts", "Kal'tsit", 6, "MEDIC", "physician", "Medic", "Medic", "RANGED", "rhodes", "Rhodes Island"),
    op("char_1012_skadi2", "Skadi the Corrupting Heart", 6, "SUPPORT", "bard", "Bard", "Supporter", "RANGED", "egir", "Ægir"),
    op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "Liberator", "Guard", "MELEE", "kazimierz", "Kazimierz"),
    op("char_293_thorns", "Thorns", 6, "WARRIOR", "lord", "Lord", "Guard", "MELEE", "iberia", "Iberia"),
    op("char_002_amiya", "Amiya", 5, "CASTER", "corecaster", "Core Caster", "Caster", "RANGED", "rhodes", "Rhodes Island"),
    op("char_291_aglina", "Angelina", 6, "SUPPORT", "slower", "Decel Binder", "Supporter", "RANGED", "siracusa", "Siracusa"),
];

/** Favourite operators: the operators page's portrait cards, rarity lines under the name bars. */
export const Operators = () => (
    <div className="w-[820px]">
        <FavouriteTiles entities={OPERATORS} />
    </div>
);

/** Favourite outfits: square tiles over the dark gradient, each with its accent hairline. */
export const Outfits = () => (
    <div className="w-[820px]">
        <FavouriteTiles
            entities={[
                skin("char_1012_skadi2@iteration#2", "Red Countess", "char_1012_skadi2", "Skadi the Corrupting Heart", 6, "SUPPORT", "Iteration Provident"),
                skin("char_293_thorns@boc#8", "Blade-cleaved Tides", "char_293_thorns", "Thorns", 6, "WARRIOR", "Bloodline of Combat"),
                skin("char_003_kalts@boc#6", "Remnant", "char_003_kalts", "Kal'tsit", 6, "MEDIC", "Bloodline of Combat"),
                skin("char_002_amiya@winter#1", "Newsgirl", "char_002_amiya", "Amiya", 5, "CASTER", "Test Collection"),
            ]}
        />
    </div>
);

/** Favourite events: wide banners, 2.1 squares across. */
export const Events = () => (
    <div className="w-[820px]">
        <FavouriteTiles entities={[event("act18d3", "Under Tides", "BRANCHLINE", 1634828400), event("act31side", "Here A People Sows", "SIDESTORY", 1722427200), event("act9mini", "Pinus Sylvestris", "MINISTORY", 1649948400)]} />
    </div>
);

/** Main-story posters with one id the game data no longer knows: the owner sees a dashed placeholder naming it. */
export const WithRemovedEntry = () => (
    <div className="w-[820px]">
        <FavouriteTiles
            entities={[
                mainStory("main_8", "Roaring Flare", "/assets/textures/spritepack/mixstory_kv_sprites_1/kv_roaring_flare.png", 8, 1, "Shatter of A Vision"),
                mainStory("main_10", "Shatterpoint", "/assets/textures/spritepack/mixstory_kv_sprites_1/kv_shatterpoint.png", 10, 2, "Shadow of A Dying Sun"),
                mainStory("main_14", "Absolved Will Be the Seekers", "/assets/textures/spritepack/mixstory_kv_sprites_0/kv_absolved_will_be_the_seekers.png", 14, 2, "Shadow of A Dying Sun"),
                { id: "main_99", entity: null, server: null },
            ]}
        />
    </div>
);
