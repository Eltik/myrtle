import { useQuery } from "@tanstack/react-query";
import { RotateCwIcon, SearchIcon, Trash2Icon } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { usePoolKind } from "#/components/tier-lists/edit/poolKinds";
import { EntityAvatar } from "#/components/tier-lists/entities";
import { useEntityLabels } from "#/components/tier-lists/kinds";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { ScrollArea } from "#/components/ui/scroll-area";
import { Spinner } from "#/components/ui/spinner";
import { type ITierEntity, type TierEntityKind, toTierEntity, UNPLACED } from "#/lib/api/tier-entities";
import { tierEntityCatalogueQueryOptions } from "#/lib/api/tier-lists";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { compactForSearch } from "#/lib/search/fuzzy";
import { cn } from "#/lib/utils";
import type { messages } from "./EntityPickerDialog.messages";
import { orderKinds } from "./shared";
import { type IPickerTarget, initialPickerKind } from "./state";

/** Tiles rendered at first and per "Show more": the enemy catalogue alone is ~1,540 entries. */
const PAGE_SIZE = 180;
/**
 * The tab the author used last, remembered for this page load only. It used
 * to live in localStorage, so a tab from an earlier session decided the first
 * open of a new one; a fresh page now opens on the grid's first allowed type.
 */
let lastKind: TierEntityKind | null = null;

interface IEntityPickerDialogProps {
    /** The cell being picked for, `null` when closed. */
    target: IPickerTarget | null;
    /** The grid's allowed types: the only tabs offered, in `ALL_ENTITY_KINDS` order. */
    kinds: readonly TierEntityKind[];
    onClose: () => void;
    onPick: (entity: ITierEntity) => void;
    onClear: () => void;
}

export function EntityPickerDialog({ target, kinds, onClose, onPick, onClear }: IEntityPickerDialogProps) {
    const t: TypedT<typeof messages> = useT("grids");
    const open = target !== null;

    return (
        <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
            <DialogPopup className="h-[min(85dvh,760px)] sm:max-w-3xl">
                <DialogHeader>
                    <DialogTitle>{target?.label ? t("picker.titleLabelled", { label: target.label }) : t("picker.title", { row: target?.row ?? 1, col: target?.col ?? 1 })}</DialogTitle>
                    <DialogDescription>{t("picker.description")}</DialogDescription>
                </DialogHeader>
                {target && <PickerBody kinds={kinds} current={target.current} onPick={onPick} />}
                <DialogFooter className="justify-between sm:justify-between">
                    <Button type="button" variant="ghost" onClick={onClear} disabled={!target?.hasPick}>
                        <Trash2Icon />
                        {t("picker.clear")}
                    </Button>
                    <DialogClose render={<Button type="button" variant="outline" />}>{t("picker.cancel")}</DialogClose>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}

function PickerBody({ kinds, current, onPick }: { kinds: readonly TierEntityKind[]; current: ITierEntity | null; onPick: (entity: ITierEntity) => void }) {
    const t: TypedT<typeof messages> = useT("grids");
    const labels = useEntityLabels();
    const server = useGamedataServer();
    const tabs = useMemo(() => orderKinds(kinds), [kinds]);
    const [kind, setKind] = useState<TierEntityKind>(() => initialPickerKind(tabs, current?.kind ?? null, lastKind));
    const [query, setQuery] = useState("");
    const [limit, setLimit] = useState(PAGE_SIZE);

    const catalogue = useQuery(tierEntityCatalogueQueryOptions(kind, server));
    const entities = useMemo(() => catalogue.data?.map((summary) => toTierEntity(summary.kind, summary.id, summary, UNPLACED)) ?? [], [catalogue.data]);
    const pool = usePoolKind(kind, entities);

    const filtered = useMemo(() => {
        const q = compactForSearch(query);
        const matched = q.length === 0 ? entities : entities.filter((entity) => pool.searchTexts(entity).some((text) => Boolean(text) && compactForSearch(text ?? "").includes(q)));
        return pool.compare ? [...matched].sort(pool.compare) : matched;
    }, [entities, pool, query]);

    // A new tab or search starts the pages over.
    // biome-ignore lint/correctness/useExhaustiveDependencies: `kind` and `query` are the triggers, not values read inside
    useEffect(() => setLimit(PAGE_SIZE), [kind, query]);

    const selectKind = (next: TierEntityKind) => {
        setKind(next);
        lastKind = next;
    };

    const visible = filtered.slice(0, limit);

    return (
        <div className="flex min-h-0 flex-1 flex-col gap-3 px-6 pb-3">
            <div className="-mx-6 overflow-x-auto px-6" role="tablist" aria-label={t("picker.kinds")}>
                <div className="flex w-max gap-1.5">
                    {tabs.map((k) => (
                        <button
                            key={k}
                            type="button"
                            role="tab"
                            aria-selected={k === kind}
                            onClick={() => selectKind(k)}
                            className={cn(
                                "inline-flex h-8 shrink-0 cursor-pointer items-center rounded-full border px-3 font-medium font-sans text-xs leading-none transition-colors",
                                k === kind ? "border-primary bg-primary text-primary-foreground" : "border-border bg-popover text-muted-foreground hover:bg-accent hover:text-foreground",
                            )}
                        >
                            {labels.plural(k)}
                        </button>
                    ))}
                </div>
            </div>

            <InputGroup>
                <InputGroupAddon>
                    <SearchIcon aria-hidden="true" />
                </InputGroupAddon>
                <InputGroupInput value={query} onChange={(e) => setQuery((e.target as HTMLInputElement).value)} placeholder={t("picker.searchPlaceholder", { kind: labels.plural(kind) })} type="search" aria-label={pool.searchLabel} />
            </InputGroup>

            <div className="relative min-h-0 flex-1">
                {catalogue.status === "pending" ? (
                    <div className="flex h-full items-center justify-center gap-2 font-sans text-muted-foreground text-sm">
                        <Spinner />
                        <span>{t("picker.loading")}</span>
                    </div>
                ) : catalogue.status === "error" ? (
                    <div className="flex h-full flex-col items-center justify-center gap-3 font-sans text-muted-foreground text-sm">
                        <span>{t("picker.error")}</span>
                        <Button type="button" variant="outline" size="sm" onClick={() => void catalogue.refetch()}>
                            <RotateCwIcon />
                            {t("picker.retry")}
                        </Button>
                    </div>
                ) : filtered.length === 0 ? (
                    <div className="flex h-full items-center justify-center font-sans text-muted-foreground text-sm">{t("picker.empty")}</div>
                ) : (
                    <ScrollArea className="h-full">
                        <ul className="m-0 grid list-none grid-cols-[repeat(auto-fill,minmax(76px,1fr))] gap-2 p-0 pe-2">
                            {visible.map((entity) => {
                                const selected = current?.key === entity.key;
                                return (
                                    <li key={entity.key}>
                                        <button
                                            type="button"
                                            onClick={() => onPick(entity)}
                                            aria-label={labels.tileLabel(entity)}
                                            aria-pressed={selected}
                                            title={labels.tileLabel(entity)}
                                            className={cn("group flex w-full cursor-pointer flex-col overflow-hidden rounded-md border bg-card text-start transition-colors hover:border-primary", selected ? "border-primary ring-2 ring-primary/40" : "border-border")}
                                        >
                                            <span className="relative flex aspect-square w-full items-center justify-center overflow-hidden bg-[oklch(0.2_0.005_285)] text-lg text-white">
                                                <EntityAvatar entity={entity} face="tile" tone="dark" />
                                            </span>
                                            <span className="line-clamp-2 min-h-[2.4em] px-1 py-1 text-center font-medium font-sans text-[10.5px] text-foreground leading-tight">{entity.name}</span>
                                        </button>
                                    </li>
                                );
                            })}
                        </ul>
                        {filtered.length > limit && (
                            <div className="flex justify-center py-3">
                                <Button type="button" variant="outline" size="sm" onClick={() => setLimit((n) => n + PAGE_SIZE)}>
                                    {t("picker.showMore", { shown: limit, total: filtered.length })}
                                </Button>
                            </div>
                        )}
                    </ScrollArea>
                )}
            </div>
        </div>
    );
}
