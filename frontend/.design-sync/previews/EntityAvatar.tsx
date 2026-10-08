import { EntityAvatar } from "frontend";
import type { ReactNode } from "react";

// The face of a tier-list tile, shared by the board, the editor, the drag ghost
// and the hover previews. An operator renders through OperatorAvatar; every
// other kind draws its own art with the kind's fit (cover, object, or an inset
// glyph inverted for the light theme); a placement the data does not know shows
// its raw id. Entities are live `/api/tier-lists/<slug>` placements mapped
// through `toTierEntity`.

const SURTR = {"key":"operator:char_350_surtr","id":"char_350_surtr","name":"Surtr","icon":"/avatar/char_350_surtr","href":"/operators/char_350_surtr","facets":{"appellation":" ","nation_id":"rhodes","nation_name":"Rhodes Island","position":"MELEE","profession":"WARRIOR","profession_name":"Guard","rarity":"6","sub_profession_id":"artsfghter","sub_profession_name":"Arts Fighter"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"operator","resolved":true,"appellation":" ","rarity":6,"profession":"WARRIOR","subProfessionId":"artsfghter","professionName":"Guard","subProfessionName":"Arts Fighter","position":"MELEE","nationId":"rhodes","nationName":"Rhodes Island"} as const;
const THORNS_SKIN = {"key":"skin:char_293_thorns@boc#8","id":"char_293_thorns@boc#8","name":"Blade-cleaved Tides","icon":"/avatar/char_293_thorns_boc%238","href":"/operators/char_293_thorns","facets":{"brand":"Bloodline of Combat","char_id":"char_293_thorns","operator":"Thorns","profession":"WARRIOR","rarity":"6"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"skin","resolved":true,"charId":"char_293_thorns","operatorName":"Thorns","rarity":6,"profession":"WARRIOR","brand":"Bloodline of Combat"} as const;
const CENTURION = {"key":"enemy:enemy_1501_demonk","id":"enemy_1501_demonk","name":"Sarkaz Centurion","icon":"/enemy-icon/enemy_1501_demonk","href":"/enemies/enemy_1501_demonk","facets":{"enemy_index":"SC","enemy_level":"BOSS"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"enemy","resolved":true,"level":"BOSS","index":"SC"} as const;
const SHATTERPOINT = {"key":"main_story:main_10","id":"main_10","name":"Shatterpoint","icon":"/assets/textures/spritepack/mixstory_kv_sprites_1/kv_shatterpoint.png","href":null,"facets":{"act":"2","act_name":"Shadow of A Dying Sun","episode":"10"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"main_story","resolved":true,"episode":10,"act":2,"actName":"Shadow of A Dying Sun"} as const;
const RHODES = {"key":"faction:rhodes","id":"rhodes","name":"Rhodes Island","icon":"/assets/textures/spritepack/ui_camp_logo_0/logo_rhodes.png","href":null,"facets":{"power_level":"nation"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"faction","resolved":true,"powerLevel":"nation"} as const;
const AGILE = {"key":"stronghold_bond:skillfulShip","id":"skillfulShip","name":"Agile","icon":"/assets/textures/ui/autochess/%5Buc%5Dautochesscommon/icon_skillfulShip.png","href":null,"facets":{"bond_type":"regular"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"stronghold_bond","resolved":true,"bondType":"regular"} as const;
const CLEMENTIA = {"key":"story_sprite:avg_npc_1382","id":"avg_npc_1382","name":"Clementia","icon":"/story-sprite-thumb/avg_npc_1382","href":null,"facets":{"face":"0.436,0.107","source":"npc"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"story_sprite","resolved":true,"source":"npc"} as const;
const PALE_SEA_NO_ART = {"key":"event:act39side","id":"act39side","name":"Exodus from the Pale Sea","icon":null,"href":null,"facets":{"display_type":"SIDESTORY","start_time":"1749124800"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"event","resolved":true,"displayType":"SIDESTORY","startTime":1749124800,"rerun":false} as const;
const UNKNOWN = {"key":"operator:char_4199_unrel","id":"char_4199_unrel","name":"char_4199_unrel","icon":null,"href":null,"facets":{},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"operator","resolved":false} as const;

/** A sized tile wrapper with a caption; `dark` is the drag ghost's always-dark box. */
const Tile = ({ children, label, dark = false, wide = false }: { children: ReactNode; label: string; dark?: boolean; wide?: boolean }) => (
    <figure className="m-0 flex flex-col items-center gap-1.5">
        <span className={`inline-flex h-16 items-center justify-center overflow-hidden rounded-lg font-sans text-lg ${dark ? "text-white" : "bg-muted text-foreground"}`} style={{ width: wide ? 134 : 64, ...(dark ? { background: "oklch(0.22 0.005 285)" } : {}) }}>
            {children}
        </span>
        <figcaption className="font-mono text-[10px] text-muted-foreground">{label}</figcaption>
    </figure>
);

/** Every art fit side by side: avatar, skin, enemy, story poster (cover), faction and bond (glyph), sprite head. */
export const KindsSweep = () => (
    <div className="flex flex-wrap gap-3 p-4">
        <Tile label="operator"><EntityAvatar entity={SURTR} /></Tile>
        <Tile label="skin"><EntityAvatar entity={THORNS_SKIN} /></Tile>
        <Tile label="enemy"><EntityAvatar entity={CENTURION} /></Tile>
        <Tile label="main_story"><EntityAvatar entity={SHATTERPOINT} /></Tile>
        <Tile label="faction"><EntityAvatar entity={RHODES} /></Tile>
        <Tile label="bond"><EntityAvatar entity={AGILE} /></Tile>
        <Tile label="story_sprite"><EntityAvatar entity={CLEMENTIA} /></Tile>
    </div>
);

/** Glyph kinds on the drag ghost's dark box: `tone="dark"` keeps the white glyph white in the light theme. */
export const DarkTone = () => (
    <div className="flex flex-wrap gap-3 p-4">
        <Tile label="faction" dark><EntityAvatar entity={RHODES} tone="dark" /></Tile>
        <Tile label="bond" dark><EntityAvatar entity={AGILE} tone="dark" /></Tile>
        <Tile label="enemy" dark><EntityAvatar entity={CENTURION} tone="dark" /></Tile>
    </div>
);

/** No art: a chip falls back to initials; a wide kind's `tile` face spells the name across the tile. */
export const NoArt = () => (
    <div className="flex flex-wrap items-start gap-3 p-4">
        <Tile label="chip"><EntityAvatar entity={PALE_SEA_NO_ART} /></Tile>
        <Tile label="tile (wide)" wide><EntityAvatar entity={PALE_SEA_NO_ART} face="tile" /></Tile>
    </div>
);

/** A placement the served data does not know (a CN id on the EN server): its raw id, never dropped. */
export const Unresolved = () => (
    <div className="p-4">
        <Tile label="unresolved"><EntityAvatar entity={UNKNOWN} /></Tile>
    </div>
);
