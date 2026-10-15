import { GridCellSheet } from "frontend";
import { type ReactNode, useEffect } from "react";

// The phone editor's cell sheet: full screen over the board, the pick drawn
// large, its label field, change / clear actions and prev / next through every
// cell. `index` drives it open; the picker nests inside it as `children`.
// Cells reuse the editor board's class-favourites fixture (live `/api/grids` data).

const CELLS = [{"label":"Favorite Vanguard","kind":"operator","id":"char_222_bpipe","entity":{"key":"operator:char_222_bpipe","id":"char_222_bpipe","name":"Bagpipe","icon":"/avatar/char_222_bpipe","href":"/operators/char_222_bpipe","facets":{"position":"MELEE","profession":"PIONEER","rarity":"6"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"operator","resolved":true,"appellation":" ","rarity":6,"profession":"PIONEER","subProfessionId":"charger","professionName":"Vanguard","subProfessionName":"Charger","position":"MELEE","nationId":"victoria","nationName":"Victoria"},"server":null},{"label":"Favorite Guard","kind":"operator","id":"char_4182_oblvns","entity":{"key":"operator:char_4182_oblvns","id":"char_4182_oblvns","name":"Sakiko Togawa","icon":"/avatar/char_4182_oblvns","href":"/operators/char_4182_oblvns","facets":{"position":"MELEE","profession":"WARRIOR","rarity":"6"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"operator","resolved":true,"appellation":" ","rarity":6,"profession":"WARRIOR","subProfessionId":"lord","professionName":"Guard","subProfessionName":"Lord","position":"MELEE","nationId":null,"nationName":null},"server":null},{"label":"Favorite Defender","kind":"operator","id":"char_2025_shu","entity":{"key":"operator:char_2025_shu","id":"char_2025_shu","name":"Shu","icon":"/avatar/char_2025_shu","href":"/operators/char_2025_shu","facets":{"position":"MELEE","profession":"TANK","rarity":"6"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"operator","resolved":true,"appellation":" ","rarity":6,"profession":"TANK","subProfessionId":"guardian","professionName":"Defender","subProfessionName":"Guardian","position":"MELEE","nationId":"yan","nationName":"Yan"},"server":null},{"label":"Favorite Medic","kind":"operator","id":"char_4163_rosesa","entity":{"key":"operator:char_4163_rosesa","id":"char_4163_rosesa","name":"Rose Salt","icon":"/avatar/char_4163_rosesa","href":"/operators/char_4163_rosesa","facets":{"position":"RANGED","profession":"MEDIC","rarity":"5"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"operator","resolved":true,"appellation":" ","rarity":5,"profession":"MEDIC","subProfessionId":"ringhealer","professionName":"Medic","subProfessionName":"Multi-target Medic","position":"RANGED","nationId":"iberia","nationName":"Iberia"},"server":null},{"label":"Favorite Sniper","kind":"operator","id":"char_113_cqbw","entity":{"key":"operator:char_113_cqbw","id":"char_113_cqbw","name":"W","icon":"/avatar/char_113_cqbw","href":"/operators/char_113_cqbw","facets":{"position":"RANGED","profession":"SNIPER","rarity":"6"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"operator","resolved":true,"appellation":" ","rarity":6,"profession":"SNIPER","subProfessionId":"aoesniper","professionName":"Sniper","subProfessionName":"Artilleryman","position":"RANGED","nationId":null,"nationName":null},"server":null},{"label":"Favorite Caster","kind":null,"id":null,"entity":null,"server":null},{"label":"Favorite Supporter","kind":"operator","id":"char_4091_ulika","entity":{"key":"operator:char_4091_ulika","id":"char_4091_ulika","name":"U-Official","icon":"/avatar/char_4091_ulika","href":"/operators/char_4091_ulika","facets":{"position":"RANGED","profession":"SUPPORT","rarity":"1"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"operator","resolved":true,"appellation":" ","rarity":1,"profession":"SUPPORT","subProfessionId":"bard","professionName":"Supporter","subProfessionName":"Bard","position":"RANGED","nationId":"rhodes","nationName":"Rhodes Island"},"server":null},{"label":"Favorite Specialist","kind":"operator","id":"char_4036_forcer","entity":{"key":"operator:char_4036_forcer","id":"char_4036_forcer","name":"Enforcer","icon":"/avatar/char_4036_forcer","href":"/operators/char_4036_forcer","facets":{"position":"MELEE","profession":"SPECIAL","rarity":"5"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"operator","resolved":true,"appellation":" ","rarity":5,"profession":"SPECIAL","subProfessionId":"pusher","professionName":"Specialist","subProfessionName":"Push Stroker","position":"MELEE","nationId":"laterano","nationName":"Laterano"},"server":null},{"label":"Main Assistant","kind":null,"id":null,"entity":null,"server":null}];

const noop = () => {};

/** Full-viewport stage; Done takes initial focus, so the ring is blurred after the open transition. */
const Stage = ({ children }: { children: ReactNode }) => {
    useEffect(() => {
        const ids = [60, 180, 400].map((ms) => setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
        return () => ids.forEach(clearTimeout);
    }, []);
    return <div className="min-h-dvh">{children}</div>;
};

/** A filled cell: the operator's art, label, change and clear. */
export const FilledCell = () => (
    <Stage>
        <GridCellSheet cells={CELLS} cols={3} index={1} onIndexChange={noop} onClose={noop} onPick={noop} onClear={noop} onLabelChange={noop} />
    </Stage>
);

/** A cell with a label and no pick: the add prompt. */
export const EmptyCell = () => (
    <Stage>
        <GridCellSheet cells={CELLS} cols={3} index={5} onIndexChange={noop} onClose={noop} onPick={noop} onClear={noop} onLabelChange={noop} />
    </Stage>
);
