import { SlidersHorizontalIcon } from "lucide-react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import type { ITierEntity, TierEntityKind } from "#/lib/api/tier-entities";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { useEntityLabels } from "../kinds";
import { type CatalogueStatus, KindPool } from "./KindPool";
import type { messages } from "./KindPool.messages";

export interface IKindCatalogue {
    entities: ITierEntity[] | undefined;
    status: CatalogueStatus;
    refetch: () => void;
}

interface IEntityPoolProps {
    /** The kinds the list offers, in the order its pool tabs show them. */
    kinds: readonly TierEntityKind[];
    /** Each offered kind's catalogue, from `GET /tier-lists/catalogue/{kind}`. */
    catalogues: Partial<Record<TierEntityKind, IKindCatalogue>>;
    placedKeys: Set<string>;
    /** How many placements of each kind are on the board, for the tab counts. */
    placedByKind: Partial<Record<TierEntityKind, number>>;
    onUnplace: (entityKey: string) => void;
    onPickerActivate: (entity: ITierEntity) => void;
    /** Opens the settings for which kinds the list offers. Absent: no Kinds button. */
    onEditKinds?: () => void;
    rootClassName?: string;
}

/**
 * The editor's pool of placeable entities: one tab per kind the list offers
 * (no tab bar when it offers one), each with its own search and filters.
 */
export function EntityPool({ kinds, catalogues, placedKeys, placedByKind, onUnplace, onPickerActivate, onEditKinds, rootClassName }: IEntityPoolProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const labels = useEntityLabels();
    const [chosen, setChosen] = useState<TierEntityKind | null>(null);
    // A kind dropped from the list's settings takes its tab with it; fall back to the first offered.
    const active: TierEntityKind = chosen && kinds.includes(chosen) ? chosen : (kinds[0] ?? "operator");
    const catalogue = catalogues[active];

    const tabs =
        kinds.length > 1 ? (
            <Tabs value={active} onValueChange={(v) => setChosen(v as TierEntityKind)}>
                <TabsList variant="underline" aria-label={t("edit.pool.tabs")} className="-mx-1 w-[calc(100%+0.5rem)] justify-start overflow-x-auto border-border border-b px-1 [-ms-overflow-style:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
                    {kinds.map((kind) => {
                        const placed = placedByKind[kind] ?? 0;
                        return (
                            <TabsTab key={kind} value={kind} className="h-8 grow-0 px-2 font-sans text-[12.5px] sm:h-8 sm:text-[12.5px]">
                                {labels.plural(kind)}
                                {placed > 0 && <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{placed}</span>}
                            </TabsTab>
                        );
                    })}
                </TabsList>
            </Tabs>
        ) : null;

    const kindsButton = onEditKinds ? (
        <Tooltip>
            <TooltipTrigger
                render={
                    <Button type="button" variant="ghost" size="xs" onClick={onEditKinds} aria-haspopup="dialog">
                        <SlidersHorizontalIcon />
                        {t("edit.pool.kinds")}
                    </Button>
                }
            />
            <TooltipContent>{t("edit.pool.kindsHint")}</TooltipContent>
        </Tooltip>
    ) : null;

    return (
        <KindPool
            // A fresh pool per kind: each kind's search and filters mean nothing to another.
            key={active}
            kind={active}
            entities={catalogue?.entities}
            status={catalogue?.status ?? "pending"}
            onRetry={catalogue?.refetch}
            placedKeys={placedKeys}
            onUnplace={onUnplace}
            onPickerActivate={onPickerActivate}
            tabs={tabs}
            headerActions={kindsButton}
            rootClassName={rootClassName}
        />
    );
}
