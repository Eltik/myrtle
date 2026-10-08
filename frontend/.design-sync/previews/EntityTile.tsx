import { EntityTile } from "frontend";
import type { CSSProperties, ReactNode } from "react";

// One placement on the tier-list board. An operator keeps its own tile; every
// other kind gets the same tile with its own art, a rarity/rank accent bar, a
// hover card naming it and a link when the site has a page for it. A wide kind
// (an event banner) spans two slots. Entities are live `/api/tier-lists/<slug>`
// placements mapped through `toTierEntity`. The stage carries `.light` (the app sets
// `.light`/`.dark` on <html>), which the tile module keys its light background on.

const SURTR = {"key":"operator:char_350_surtr","id":"char_350_surtr","name":"Surtr","icon":"/avatar/char_350_surtr","href":"/operators/char_350_surtr","facets":{"appellation":" ","nation_id":"rhodes","nation_name":"Rhodes Island","position":"MELEE","profession":"WARRIOR","profession_name":"Guard","rarity":"6","sub_profession_id":"artsfghter","sub_profession_name":"Arts Fighter"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"operator","resolved":true,"appellation":" ","rarity":6,"profession":"WARRIOR","subProfessionId":"artsfghter","professionName":"Guard","subProfessionName":"Arts Fighter","position":"MELEE","nationId":"rhodes","nationName":"Rhodes Island"} as const;
const THORNS_SKIN = {"key":"skin:char_293_thorns@boc#8","id":"char_293_thorns@boc#8","name":"Blade-cleaved Tides","icon":"/avatar/char_293_thorns_boc%238","href":"/operators/char_293_thorns","facets":{"brand":"Bloodline of Combat","char_id":"char_293_thorns","operator":"Thorns","profession":"WARRIOR","rarity":"6"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"skin","resolved":true,"charId":"char_293_thorns","operatorName":"Thorns","rarity":6,"profession":"WARRIOR","brand":"Bloodline of Combat"} as const;
const CENTURION = {"key":"enemy:enemy_1501_demonk","id":"enemy_1501_demonk","name":"Sarkaz Centurion","icon":"/enemy-icon/enemy_1501_demonk","href":"/enemies/enemy_1501_demonk","facets":{"enemy_index":"SC","enemy_level":"BOSS"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"enemy","resolved":true,"level":"BOSS","index":"SC"} as const;
const SHATTERPOINT = {"key":"main_story:main_10","id":"main_10","name":"Shatterpoint","icon":"/assets/textures/spritepack/mixstory_kv_sprites_1/kv_shatterpoint.png","href":null,"facets":{"act":"2","act_name":"Shadow of A Dying Sun","episode":"10"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"main_story","resolved":true,"episode":10,"act":2,"actName":"Shadow of A Dying Sun"} as const;
const RHODES = {"key":"faction:rhodes","id":"rhodes","name":"Rhodes Island","icon":"/assets/textures/spritepack/ui_camp_logo_0/logo_rhodes.png","href":null,"facets":{"power_level":"nation"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"faction","resolved":true,"powerLevel":"nation"} as const;
const AGILE = {"key":"stronghold_bond:skillfulShip","id":"skillfulShip","name":"Agile","icon":"/assets/textures/ui/autochess/%5Buc%5Dautochesscommon/icon_skillfulShip.png","href":null,"facets":{"bond_type":"regular"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"stronghold_bond","resolved":true,"bondType":"regular"} as const;
const CLEMENTIA = {"key":"story_sprite:avg_npc_1382","id":"avg_npc_1382","name":"Clementia","icon":"/story-sprite-thumb/avg_npc_1382","href":null,"facets":{"face":"0.436,0.107","source":"npc"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"story_sprite","resolved":true,"source":"npc"} as const;
const UNKNOWN = {"key":"operator:char_4199_unrel","id":"char_4199_unrel","name":"char_4199_unrel","icon":null,"href":null,"facets":{},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"operator","resolved":false} as const;
const PALE_SEA = {"key":"event:act39side","id":"act39side","name":"Exodus from the Pale Sea","icon":"/event-image/act39side","href":null,"facets":{"display_type":"SIDESTORY","start_time":"1749124800","type":"TYPE_ACT9D0"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"event","resolved":true,"displayType":"SIDESTORY","startTime":1749124800,"rerun":false} as const;
const SLUG = {"key":"enemy:enemy_1007_slime","id":"enemy_1007_slime","name":"Originium Slug","icon":"/enemy-icon/enemy_1007_slime","href":"/enemies/enemy_1007_slime","facets":{"enemy_index":"B1","enemy_level":"NORMAL"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"enemy","resolved":true,"level":"NORMAL","index":"B1"} as const;

/** The custom properties `TierListBoard` sets on the board, so a tile measures as it does on the page. */
const BOARD_VARS = { "--op-size": "64px", "--op-gap": "6px", "--op-radius": "8px" } as CSSProperties;

const Row = ({ children }: { children: ReactNode }) => (
    <div className="light p-4" style={BOARD_VARS}>
        <div className="flex flex-wrap items-center gap-1.5">{children}</div>
    </div>
);

/** A mixed row: operator, skin, boss and normal enemy, story poster, story sprite. */
export const MixedRow = () => (
    <Row>
        <EntityTile entity={SURTR} />
        <EntityTile entity={THORNS_SKIN} />
        <EntityTile entity={CENTURION} />
        <EntityTile entity={SLUG} />
        <EntityTile entity={SHATTERPOINT} />
        <EntityTile entity={CLEMENTIA} />
    </Row>
);

/** An event banner takes two slots so a row of mixed kinds keeps one height. */
export const WideEvent = () => (
    <Row>
        <EntityTile entity={PALE_SEA} />
        <EntityTile entity={SHATTERPOINT} />
    </Row>
);

/** Glyph kinds: a faction and a Stronghold Protocol bond, inset and theme-inverted. */
export const Glyphs = () => (
    <Row>
        <EntityTile entity={RHODES} />
        <EntityTile entity={AGILE} />
    </Row>
);

/** A placement the served data does not know: its raw id, with the neutral accent. */
export const Unresolved = () => (
    <Row>
        <EntityTile entity={UNKNOWN} />
        <EntityTile entity={SURTR} />
    </Row>
);
