import { CircleSlash2Icon, FilterXIcon, Maximize2Icon, SearchIcon } from "lucide-react";
import { useCallback, useMemo, useRef, useState } from "react";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { Field, FieldLabel } from "#/components/ui/field";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { Kicker } from "#/components/ui/kicker";
import { ScrollArea } from "#/components/ui/scroll-area";
import { Switch } from "#/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "#/components/ui/tooltip";
import type { ITierOperator } from "#/lib/api/tier-lists";
import { compactForSearch } from "#/lib/search/fuzzy";
import { cn, formatProfession } from "#/lib/utils";
import type { IOperatorIndexEntry, OperatorProfession } from "#/types/operators";
import { indexEntryToTierOperator } from "../shared";
import { hasOperatorDrag, readOperatorDrag } from "./dnd";
import { useAnyDragLifted, usePoolIsOver } from "./drag-controller";
import { EditableOpTile } from "./EditableOpTile";
import styles from "./Editor.module.css";

const RARITY_OPTIONS = [6, 5, 4, 3, 2, 1] as const;
const CLASS_OPTIONS: OperatorProfession[] = ["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER", "MEDIC", "SUPPORT", "SPECIAL"];

interface IOperatorPoolProps {
    operators: IOperatorIndexEntry[];
    placedIds: Set<string>;
    onUnplace: (operatorId: string) => void;
    onPickerActivate: (operator: ITierOperator) => void;
    /** Override the root container classes. Use when mounting inside a flex/grid layout that needs `flex-1 min-h-0` instead of `h-full`. */
    rootClassName?: string;
}

export function OperatorPool({ operators, placedIds, onUnplace, onPickerActivate, rootClassName }: IOperatorPoolProps) {
    const [query, setQuery] = useState("");
    const [rarities, setRarities] = useState<number[]>([]);
    const [classes, setClasses] = useState<OperatorProfession[]>([]);
    const [hideUsed, setHideUsed] = useState(true);
    const [expandedOpen, setExpandedOpen] = useState(false);
    const [mouseDragOver, setMouseDragOver] = useState(false);
    const dropRef = useRef<HTMLElement | null>(null);
    const touchIsOver = usePoolIsOver();
    const anyDragLifted = useAnyDragLifted();
    const isDragOver = touchIsOver || mouseDragOver;

    const filtered = useMemo(() => {
        const q = compactForSearch(query);
        return operators
            .filter((op) => !op.isNotObtainable)
            .filter((op) => {
                if (rarities.length > 0 && !rarities.includes(op.rarity)) return false;
                if (classes.length > 0 && !classes.includes(op.profession)) return false;
                if (hideUsed && placedIds.has(op.id)) return false;
                if (q.length === 0) return true;
                return compactForSearch(op.name).includes(q) || compactForSearch(op.appellation).includes(q);
            })
            .sort((a, b) => {
                if (a.rarity !== b.rarity) return b.rarity - a.rarity;
                return a.name.localeCompare(b.name);
            });
    }, [operators, query, rarities, classes, hideUsed, placedIds]);

    const totalAvailable = useMemo(() => operators.filter((op) => !op.isNotObtainable).length, [operators]);
    const hasFilters = query.length > 0 || rarities.length > 0 || classes.length > 0 || !hideUsed;

    const handleOver = useCallback((e: React.DragEvent) => {
        if (!hasOperatorDrag(e)) return;
        e.preventDefault();
        e.dataTransfer.dropEffect = "move";
        setMouseDragOver(true);
    }, []);

    const handleLeave = useCallback((e: React.DragEvent) => {
        const related = e.relatedTarget as Node | null;
        if (related && dropRef.current?.contains(related)) return;
        setMouseDragOver(false);
    }, []);

    const handleDrop = useCallback(
        (e: React.DragEvent) => {
            setMouseDragOver(false);
            const payload = readOperatorDrag(e);
            if (!payload) return;
            e.preventDefault();
            onUnplace(payload.operatorId);
        },
        [onUnplace],
    );

    const handleClearFilters = useCallback(() => {
        setQuery("");
        setRarities([]);
        setClasses([]);
        setHideUsed(true);
    }, []);

    return (
        <TooltipProvider delay={300}>
            <div className={cn("flex h-full min-h-0 flex-col gap-3 rounded-xl border border-border bg-card p-3 shadow-[0_1px_2px_oklch(0_0_0/0.04)]", rootClassName)}>
                <header className="flex items-baseline justify-between gap-2">
                    <div className="min-w-0">
                        <Kicker className="mb-0.5">Operator pool</Kicker>
                        <p className="m-0 font-mono text-[10.5px] text-muted-foreground tabular-nums">
                            <span className="font-bold text-foreground">{filtered.length}</span>
                            <span className="opacity-70"> / {totalAvailable}</span>
                        </p>
                    </div>
                    <div className="flex shrink-0 items-center gap-0.5">
                        {hasFilters && (
                            <Button type="button" variant="ghost" size="xs" onClick={handleClearFilters} aria-label="Clear pool filters">
                                <FilterXIcon />
                                Clear
                            </Button>
                        )}
                        <Tooltip>
                            <TooltipTrigger
                                render={
                                    <Button type="button" variant="outline" size="xs" onClick={() => setExpandedOpen(true)} aria-haspopup="dialog" aria-expanded={expandedOpen}>
                                        <Maximize2Icon />
                                        Expand
                                    </Button>
                                }
                            />
                            <TooltipContent>Open a larger pool view with all filters</TooltipContent>
                        </Tooltip>
                    </div>
                </header>

                <Field>
                    <FieldLabel className="sr-only">Search operators</FieldLabel>
                    <InputGroup>
                        <InputGroupAddon>
                            <SearchIcon aria-hidden="true" />
                        </InputGroupAddon>
                        <InputGroupInput value={query} onChange={(e) => setQuery((e.target as HTMLInputElement).value)} placeholder="Search by name…" type="search" aria-label="Search operators" />
                    </InputGroup>
                </Field>

                <section
                    ref={dropRef}
                    data-tl-drop-pool=""
                    className="relative isolate flex min-h-32 flex-1 flex-col overflow-hidden rounded-lg border border-border/70 border-dashed bg-muted/20 transition-colors"
                    data-drag-over={isDragOver || undefined}
                    style={isDragOver ? { borderStyle: "solid", borderColor: "var(--ring)", background: "color-mix(in srgb, var(--ring) 8%, transparent)" } : undefined}
                    onDragOver={handleOver}
                    onDragLeave={handleLeave}
                    onDrop={handleDrop}
                    aria-label="Drag operators here to unplace them"
                >
                    {isDragOver && (
                        <div className="pointer-events-none absolute inset-0 z-10 flex items-center justify-center">
                            <Badge variant="destructive" size="lg" className="gap-1.5 shadow-md">
                                <CircleSlash2Icon className="size-3.5" />
                                Drop to unplace
                            </Badge>
                        </div>
                    )}

                    {filtered.length === 0 ? (
                        <Empty className="py-8">
                            <EmptyHeader>
                                <EmptyMedia variant="icon">
                                    <SearchIcon />
                                </EmptyMedia>
                                <EmptyTitle className="text-base">No matches</EmptyTitle>
                                <EmptyDescription>Try a different name or relax the filters.</EmptyDescription>
                            </EmptyHeader>
                        </Empty>
                    ) : (
                        <ScrollArea scrollFade scrollbarGutter viewportClassName="p-2">
                            <ul className={styles.poolGrid} aria-label="Available operators">
                                {filtered.map((entry) => {
                                    const placed = placedIds.has(entry.id);
                                    return (
                                        <li key={entry.id} className="contents">
                                            <EditableOpTile operator={indexEntryToTierOperator(entry)} placed={placed} onActivate={(op) => onPickerActivate(op)} />
                                        </li>
                                    );
                                })}
                            </ul>
                        </ScrollArea>
                    )}
                </section>

                <p className="m-0 font-sans text-muted-foreground text-xs leading-snug">{anyDragLifted ? <span className="font-medium text-foreground">Drop on a tier to place, or release here to unplace.</span> : <>Drag a tile onto a tier, or tap to pick one.</>}</p>
            </div>

            <Dialog open={expandedOpen} onOpenChange={setExpandedOpen}>
                <DialogPopup className="flex max-h-[min(820px,calc(100dvh-3rem))] w-[min(960px,calc(100vw-2rem))] flex-col gap-0 p-0 sm:max-w-none">
                    <DialogHeader className="px-6 pt-5 pb-3">
                        <DialogTitle>Operator pool</DialogTitle>
                        <DialogDescription>
                            <span className="font-mono text-foreground tabular-nums">{filtered.length}</span> of <span className="font-mono tabular-nums">{totalAvailable}</span> available. Tap a tile to place it on a tier.
                        </DialogDescription>
                    </DialogHeader>

                    <div className="flex flex-col gap-3 border-border border-y bg-muted/30 px-6 py-3">
                        <Field>
                            <FieldLabel className="sr-only">Search operators</FieldLabel>
                            <InputGroup>
                                <InputGroupAddon>
                                    <SearchIcon aria-hidden="true" />
                                </InputGroupAddon>
                                <InputGroupInput value={query} onChange={(e) => setQuery((e.target as HTMLInputElement).value)} placeholder="Search by name…" type="search" aria-label="Search operators" autoFocus />
                            </InputGroup>
                        </Field>

                        <div className="flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center sm:gap-x-4 sm:gap-y-2">
                            <Field className="gap-1.5 sm:flex-row sm:items-center sm:gap-2">
                                <FieldLabel className="whitespace-nowrap font-bold font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.16em]">Rarity</FieldLabel>
                                <div className="-mx-1 flex overflow-x-auto px-1 [-ms-overflow-style:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
                                    <ToggleGroup value={rarities.map(String)} onValueChange={(v) => setRarities((v as string[]).map(Number))} aria-label="Filter by rarity" multiple variant="outline" size="sm" className="flex-nowrap">
                                        {RARITY_OPTIONS.map((r) => (
                                            <ToggleGroupItem key={r} value={String(r)} aria-label={`${r} star`} className="shrink-0 font-mono tabular-nums [&:not([data-pressed])]:opacity-55">
                                                {r}★
                                            </ToggleGroupItem>
                                        ))}
                                    </ToggleGroup>
                                </div>
                            </Field>

                            <Field className="gap-1.5 sm:flex-row sm:items-center sm:gap-2">
                                <FieldLabel className="whitespace-nowrap font-bold font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.16em]">Class</FieldLabel>
                                <div className="-mx-1 flex overflow-x-auto px-1 [-ms-overflow-style:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
                                    <ToggleGroup value={classes} onValueChange={(v) => setClasses(v as OperatorProfession[])} aria-label="Filter by class" multiple variant="outline" size="sm" className="flex-nowrap">
                                        {CLASS_OPTIONS.map((c) => (
                                            <Tooltip key={c}>
                                                <TooltipTrigger
                                                    render={
                                                        <ToggleGroupItem value={c} aria-label={formatProfession(c)} className="shrink-0 px-1.5 [&:not([data-pressed])>img]:opacity-40">
                                                            <ClassIcon profession={c} size={16} />
                                                        </ToggleGroupItem>
                                                    }
                                                />
                                                <TooltipContent>{formatProfession(c)}</TooltipContent>
                                            </Tooltip>
                                        ))}
                                    </ToggleGroup>
                                </div>
                            </Field>

                            <label className="inline-flex cursor-pointer items-center justify-between gap-2 rounded-md px-1.5 py-1 text-foreground transition-colors hover:bg-accent/40 sm:ml-auto sm:justify-start" htmlFor="pool-hide-used-dialog">
                                <span className="flex flex-col">
                                    <span className="whitespace-nowrap font-medium font-sans text-[12.5px] leading-none">Hide already placed</span>
                                    <span className="mt-1 whitespace-nowrap text-[11px] text-muted-foreground leading-none">{placedIds.size > 0 ? `${placedIds.size} placed so far` : "Nothing placed yet"}</span>
                                </span>
                                <Switch id="pool-hide-used-dialog" checked={hideUsed} onCheckedChange={setHideUsed} />
                            </label>
                        </div>
                    </div>

                    <div className="min-h-0 flex-1 overflow-hidden px-6 py-3">
                        {filtered.length === 0 ? (
                            <Empty className="py-16">
                                <EmptyHeader>
                                    <EmptyMedia variant="icon">
                                        <SearchIcon />
                                    </EmptyMedia>
                                    <EmptyTitle className="text-base">No matches</EmptyTitle>
                                    <EmptyDescription>Try a different name or relax the filters.</EmptyDescription>
                                </EmptyHeader>
                            </Empty>
                        ) : (
                            <ScrollArea scrollFade={!anyDragLifted} scrollbarGutter viewportClassName="pr-2">
                                <ul className={styles.poolGrid} aria-label="Available operators">
                                    {filtered.map((entry) => {
                                        const placed = placedIds.has(entry.id);
                                        return (
                                            <li key={entry.id} className="contents">
                                                <Tooltip>
                                                    <TooltipTrigger
                                                        render={
                                                            <EditableOpTile
                                                                operator={indexEntryToTierOperator(entry)}
                                                                placed={placed}
                                                                onActivate={(op) => {
                                                                    setExpandedOpen(false);
                                                                    onPickerActivate(op);
                                                                }}
                                                            />
                                                        }
                                                    />
                                                    <TooltipContent>
                                                        <span className="font-sans font-semibold text-xs">{entry.name}</span>
                                                        {placed ? <span className="ms-1.5 font-mono text-[10px] uppercase tracking-wider opacity-70">Placed</span> : null}
                                                    </TooltipContent>
                                                </Tooltip>
                                            </li>
                                        );
                                    })}
                                </ul>
                            </ScrollArea>
                        )}
                    </div>

                    <DialogFooter className="justify-between border-border border-t px-6 py-3 sm:justify-between">
                        <Button type="button" variant="ghost" size="sm" onClick={handleClearFilters} disabled={!hasFilters}>
                            <FilterXIcon />
                            Clear filters
                        </Button>
                        <DialogClose render={<Button type="button" />}>Done</DialogClose>
                    </DialogFooter>
                </DialogPopup>
            </Dialog>
        </TooltipProvider>
    );
}
