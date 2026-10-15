import { CellHeroArt } from "frontend";
import type { ReactNode } from "react";

// A picked cell's art drawn large: the full-screen cell viewer (`viewer`) and
// the phone cell editor (`sheet`). An operator or skin tries its full art with
// the tile art showing until it loads; every other kind draws its tile art in a
// large frame (square, or wide for an event banner). Entities are live
// `/api/grids` cells mapped through `toTierEntity`.

const LEMUEN = {"key":"operator:char_4193_lemuen","id":"char_4193_lemuen","name":"Lemuen","icon":"/avatar/char_4193_lemuen","href":"/operators/char_4193_lemuen","facets":{"nation_id":"laterano","nation_name":"Laterano","position":"RANGED","profession":"SNIPER","profession_name":"Sniper","rarity":"6","sub_profession_id":"longrange","sub_profession_name":"Deadeye"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"operator","resolved":true,"appellation":null,"rarity":6,"profession":"SNIPER","subProfessionId":"longrange","professionName":"Sniper","subProfessionName":"Deadeye","position":"RANGED","nationId":"laterano","nationName":"Laterano"} as const;

const JUDGMENT_DAY = {"key":"skin:char_300_phenxi@boc#9","id":"char_300_phenxi@boc#9","name":"Judgment Day","icon":"/avatar/char_300_phenxi_boc%239","href":"/operators/char_300_phenxi","facets":{"brand":"Bloodline of Combat","char_id":"char_300_phenxi","operator":"Fiammetta","profession":"SNIPER","rarity":"6"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"skin","resolved":true,"charId":"char_300_phenxi","operatorName":"Fiammetta","rarity":6,"profession":"SNIPER","brand":"Bloodline of Combat"} as const;

const IDIDA = {"key":"story_sprite:avg_npc_1793","id":"avg_npc_1793","name":"Idida","icon":"/story-sprite-thumb/avg_npc_1793","href":null,"facets":{"face":"0.492,0.132","source":"npc"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"story_sprite","resolved":true,"source":"npc"} as const;

const PALE_SEA = {"key":"event:act39side","id":"act39side","name":"Exodus from the Pale Sea","icon":"/event-image/act39side","href":null,"facets":{"display_type":"SIDESTORY","start_time":"1749124800","type":"TYPE_ACT9D0"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"event","resolved":true,"displayType":"SIDESTORY","startTime":1749124800,"rerun":false} as const;

/** The viewer's stage: dark, full-bleed, a definite height for the art to fill. */
const ViewerStage = ({ children }: { children: ReactNode }) => <div className="w-full bg-[oklch(0.13_0.004_285)] p-6" style={{ height: 560 }}>{children}</div>;
/** The sheet's preview well. */
const SheetStage = ({ children }: { children: ReactNode }) => <div className="h-80 w-full max-w-md bg-[oklch(0.13_0.004_285)] p-4">{children}</div>;

/** An operator in the viewer: the E2 full art. */
export const OperatorViewer = () => (
    <ViewerStage>
        <CellHeroArt entity={LEMUEN} server={null} size="viewer" />
    </ViewerStage>
);

/** An outfit in the viewer: the skinpack art. */
export const SkinViewer = () => (
    <ViewerStage>
        <CellHeroArt entity={JUDGMENT_DAY} server={null} size="viewer" />
    </ViewerStage>
);

/** A story character: no full art, the head-and-shoulders tile in a square frame. */
export const StorySpriteSheet = () => (
    <SheetStage>
        <CellHeroArt entity={IDIDA} server={null} size="sheet" />
    </SheetStage>
);

/** An event: its banner in the wide frame. */
export const EventSheet = () => (
    <SheetStage>
        <CellHeroArt entity={PALE_SEA} server={null} size="sheet" />
    </SheetStage>
);
