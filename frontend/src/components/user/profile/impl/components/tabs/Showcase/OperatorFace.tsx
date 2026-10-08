import { useState } from "react";
import { CampIcon, ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { RARITY_BLUR_COLORS, RARITY_COLORS } from "#/components/operators/list/impl/constants";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { getPortraitById, rarityToNumber } from "#/lib/utils";
import type { OperatorProfession } from "#/types/operators";

/** What a face reads off an operator: a placed favourite or an operators index row. */
export interface IFaceOperator {
    id: string;
    name: string;
    rarity: number;
    profession: OperatorProfession;
    nationId: string | null;
}

/** The operators page's card in small: the faction mark faint behind the portrait, the name and class on a frosted bar, the rarity line under it. */
export function OperatorFace({ entity, server }: { entity: IFaceOperator; server?: string }) {
    const rarity = rarityToNumber(entity.rarity);
    return (
        <>
            {entity.nationId && <CampIcon groupId={entity.nationId} size={240} className="pointer-events-none absolute -top-3 -left-8 max-w-none opacity-5 transition-opacity group-hover:opacity-10" />}
            <PortraitArt entity={entity} server={server} />
            <span aria-hidden="true" className="absolute inset-x-0 bottom-0 z-10 flex h-9 items-end gap-1 bg-background/80 px-1.5 pb-1.5 backdrop-blur-sm">
                <span className="line-clamp-2 min-w-0 flex-1 font-bold text-[10px] uppercase leading-tight opacity-70 transition-opacity group-hover:opacity-100 sm:text-[11px]">{entity.name}</span>
                <ClassIcon profession={entity.profession} size={160} className="size-4 shrink-0" />
            </span>
            <span aria-hidden="true" className="absolute inset-x-0 bottom-0 z-10 h-0.5" style={{ backgroundColor: RARITY_COLORS[rarity] }} />
            <span aria-hidden="true" className="absolute inset-x-0 -bottom-0.5 z-10 h-1 blur-sm" style={{ backgroundColor: RARITY_BLUR_COLORS[rarity] }} />
        </>
    );
}

/** An operator's portrait (180 x 360), falling back to the square avatar where the extract has no portrait. */
function PortraitArt({ entity, server }: { entity: IFaceOperator; server?: string }) {
    const [failed, setFailed] = useState(false);
    if (failed) return <OperatorAvatar charId={entity.id} name={entity.name} server={server} />;
    return <img src={getPortraitById(entity.id, server)} alt="" aria-hidden="true" loading="lazy" decoding="async" draggable={false} onError={() => setFailed(true)} className="absolute inset-0 block h-full w-full object-contain transition-transform duration-150 group-hover:scale-105" />;
}
