import { EntityPickerDialog } from "frontend";
import { type ReactNode, useEffect } from "react";

// The grid editor's picker: one tab per allowed type, a search box, facet
// filters and a page of tiles. The catalogue loads through a server function,
// stubbed in the design bundle, so the body renders the real load-failure
// branch (message + retry) under the real header, tabs and footer.

/** Lemuen, the cell's current pick (live `/api/grids` data mapped through `toTierEntity`). */
const LEMUEN = {"key":"operator:char_4193_lemuen","id":"char_4193_lemuen","name":"Lemuen","icon":"/avatar/char_4193_lemuen","href":"/operators/char_4193_lemuen","facets":{"position":"RANGED","profession":"SNIPER","rarity":"6"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"operator","resolved":true,"appellation":null,"rarity":6,"profession":"SNIPER","subProfessionId":"longrange","professionName":"Sniper","subProfessionName":"Deadeye","position":"RANGED","nationId":"laterano","nationName":"Laterano"};

const noop = () => {};

/** Full-viewport stage; the search box takes initial focus, blurred after the open transition so its red ring does not ship. */
const Stage = ({ children }: { children: ReactNode }) => {
    useEffect(() => {
        const ids = [60, 180, 400].map((ms) => setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
        return () => ids.forEach(clearTimeout);
    }, []);
    return <div className="min-h-dvh">{children}</div>;
};

/** A labelled cell that already holds a pick: the title names the label, Clear is live. */
export const ChangingAPick = () => (
    <Stage>
        <EntityPickerDialog target={{ index: 0, row: 1, col: 1, label: "Favorite Char Top 1", current: LEMUEN, hasPick: true }} kinds={["operator", "skin", "story_sprite"]} onClose={noop} onPick={noop} onClear={noop} />
    </Stage>
);

/** An unlabelled empty cell on a grid with many allowed types: the title names the position, Clear is off, the tab row scrolls. */
export const EmptyCellManyKinds = () => (
    <Stage>
        <EntityPickerDialog target={{ index: 7, row: 2, col: 2, label: "", current: null, hasPick: false }} kinds={["operator", "subclass", "event", "faction", "stronghold_bond", "skin", "integrated_strategies", "story_sprite"]} onClose={noop} onPick={noop} onClear={noop} />
    </Stage>
);
