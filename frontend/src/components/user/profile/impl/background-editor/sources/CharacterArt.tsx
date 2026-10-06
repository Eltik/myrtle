import { memo, useCallback, useMemo } from "react";
import { EntityPickerBody } from "#/components/grids/EntityPickerDialog";
import { entityKey, type ITierEntity } from "#/lib/api/tier-entities";
import { BACKGROUND_KINDS } from "../../background";
import type { IPickedArt } from "../ArtBrowser";

interface ICharacterArtProps {
    /** The art the draft shows, so its outfit or operator shows as picked. */
    selected: IPickedArt | null;
    onPick: (kind: (typeof BACKGROUND_KINDS)[number], id: string) => void;
}

/**
 * The Character art source: the tier-list entity picker over outfits and operators, with
 * its own kind tabs, search, rarity and class filters and square tiles. It is the grids'
 * shared picker, so it is used as it is rather than restyled from here. Its `selected` set
 * is keyed on the picked kind and id alone, so a new crop or zoom does not hand the picker
 * a new set.
 */
export const CharacterArt = memo(function CharacterArt({ selected, onPick }: ICharacterArtProps) {
    const kind = selected?.kind;
    const id = selected?.id;
    const picked = useMemo(() => new Set(id !== undefined && (kind === "skin" || kind === "operator") ? [entityKey(kind, id)] : []), [kind, id]);
    const pick = useCallback(
        (entity: ITierEntity) => {
            if (entity.kind === "skin" || entity.kind === "operator") onPick(entity.kind, entity.id);
        },
        [onPick],
    );
    return <EntityPickerBody kinds={BACKGROUND_KINDS} current={null} selected={picked} onPick={pick} />;
});
