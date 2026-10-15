import { CharacterArt } from "frontend";
import { useState } from "react";

// CharacterArt is the background editor's "Character art" source: the grids'
// shared entity picker over outfits and operators (kind tabs, search, rarity
// and class filters, square tiles), used as it is. It loads the tier-entity
// catalogue through a server function the design bundle stubs to fail, so
// the honest render here is the picker's own load-failure state.

type Picked = { kind: "skin" | "operator" | "archive_pic" | "story_cg" | "story_scene"; id: string } | null;

function Stage({ initial }: { initial: Picked }) {
    const [selected, setSelected] = useState<Picked>(initial);
    return (
        <div style={{ width: 820, height: 520 }} className="flex flex-col">
            <CharacterArt selected={selected as never} onPick={(kind, id) => setSelected({ kind, id })} />
        </div>
    );
}

/** Opened with nothing picked. */
export const NothingPicked = () => <Stage initial={null} />;

/** Opened on an outfit background. */
export const OutfitPicked = () => <Stage initial={{ kind: "skin", id: "char_1012_skadi2@iteration#2" }} />;
