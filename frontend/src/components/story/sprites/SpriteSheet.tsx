import { Link } from "@tanstack/react-router";
import { DownloadIcon } from "lucide-react";
import type React from "react";
import { useCallback, useMemo, useRef, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Sheet, SheetClose, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { Skeleton } from "#/components/ui/skeleton";
import { useMediaQuery } from "#/hooks/use-media-query";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { StorySpriteDetail } from "#/types/generated/StorySpriteDetail";
import type { StorySpriteNameDetail } from "#/types/generated/StorySpriteNameDetail";
import type { StorySpriteVariant } from "#/types/generated/StorySpriteVariant";
import type { messages } from "./CharactersTab.messages";
import { useSpriteDetail } from "./data";
import { CELL_CROP, GRID_KEYS, groupVariants, initialVariant, lineCount, primaryName, stepCell } from "./gallery";
import { NameChip, StrayNames } from "./NameChip";
import { SpriteFigure } from "./SpriteCard";
import { downloadSheet, readThemeColours } from "./sheetPng";
import { spriteThumbUrl } from "./thumb";

type SpritesT = TypedT<typeof messages>;

/** Placeholder cells while a sheet loads. */
const SKELETON_CELLS = 8;

/** The sheet itself: identity, the large preview, the expression grid, the names and the stories. */
function SpriteSheetBody({ detail }: { detail: StorySpriteDetail }): React.ReactElement {
    const t: SpritesT = useT("story");
    const f = useFormatters();
    const server = useGamedataServer();
    const entry = detail.sprite;
    const name = primaryName(entry);
    const variants = detail.variants;
    // Each group with where it starts in the flat order the grid draws in, so
    // an arrow key and a click agree on a cell's index.
    const groups = useMemo(() => {
        let start = 0;
        return groupVariants(variants).map((g) => {
            const section = { ...g, start };
            start += g.variants.length;
            return section;
        });
    }, [variants]);
    const flat = useMemo(() => groups.flatMap((g) => g.variants), [groups]);
    const nameDetails = useMemo(() => {
        const byName = new Map<string, StorySpriteNameDetail>();
        for (const d of detail.names ?? []) if (!byName.has(d.name)) byName.set(d.name, d);
        return byName;
    }, [detail.names]);
    const [selected, setSelected] = useState(() => {
        const at = initialVariant(variants, entry.thumb?.key);
        return flat.indexOf(variants[at] as StorySpriteVariant);
    });
    const current = flat[Math.max(0, selected)] ?? flat[0];
    const cells = useRef<(HTMLButtonElement | null)[]>([]);
    const [busy, setBusy] = useState(false);
    const [failed, setFailed] = useState(false);
    // Said only when the canvas limits forced a smaller scale; extra columns are silent.
    const [scaled, setScaled] = useState(false);

    const onKeyDown = useCallback(
        (e: React.KeyboardEvent<HTMLDivElement>) => {
            if (!GRID_KEYS.has(e.key)) return;
            const rects = cells.current.map((el) => el?.getBoundingClientRect() ?? { left: 0, top: 0, width: 0, height: 0 });
            const next = stepCell(rects, Math.max(0, selected), e.key);
            e.preventDefault();
            setSelected(next);
            cells.current[next]?.focus();
        },
        [selected],
    );

    const download = useCallback(async () => {
        setBusy(true);
        setFailed(false);
        setScaled(false);
        try {
            const result = await downloadSheet({ base: entry.base, title: name, variants: flat, colours: readThemeColours() });
            setScaled(result.note === "halfSize");
        } catch {
            setFailed(true);
        } finally {
            setBusy(false);
        }
    }, [entry.base, name, flat]);

    return (
        <div className="grid gap-5 md:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
            <div className="flex min-w-0 flex-col gap-3">
                {current ? (
                    <div className="relative overflow-hidden rounded-xl border border-border bg-secondary/40">
                        <SpriteFigure key={current.key} variant={current} name={entry.base} crop={null} className="aspect-square w-full" />
                        <span className="absolute top-2 left-2 rounded-md bg-background/80 px-1.5 py-0.5 font-mono text-[11px] text-foreground tabular-nums backdrop-blur-sm">{current.key}</span>
                        <span className="absolute right-2 bottom-2 rounded-md bg-background/80 px-1.5 py-0.5 font-mono text-[10.5px] text-muted-foreground backdrop-blur-sm">
                            {t("sprites.sheet.uses", { count: current.uses })}
                            {current.wholeBody ? ` · ${t("sprites.sheet.wholeBody")}` : ""}
                        </span>
                    </div>
                ) : null}
                <div className="flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[11px] text-muted-foreground">
                    <span>{t("sprites.sheet.folder", { base: entry.base })}</span>
                    {entry.variant ? <span>{t("sprites.sheet.variant", { variant: entry.variant })}</span> : null}
                    {entry.firstSeen ? <span>{t("sprites.sheet.firstSeen", { date: f.date(entry.firstSeen * 1000) })}</span> : null}
                    {entry.charId ? (
                        <Link to="/operators/$id" params={{ id: entry.charId }} className="text-primary underline-offset-2 hover:underline max-sm:min-h-11 max-sm:leading-11">
                            {t("sprites.sheet.operator")}
                        </Link>
                    ) : null}
                </div>
                <section>
                    <h3 className="mt-1 mb-1.5 font-sans font-semibold text-[12px] text-muted-foreground uppercase tracking-[0.06em]">{t("sprites.sheet.names")}</h3>
                    {entry.names.length === 0 ? (
                        <p className="m-0 font-sans text-[12.5px] text-muted-foreground">{t("sprites.sheet.noNames")}</p>
                    ) : (
                        <ul className="m-0 flex list-none flex-wrap gap-1.5 p-0">
                            {entry.names.map((n, i) => (
                                <li key={n.name}>
                                    <NameChip name={n.name} count={n.count} primary={i === 0} detail={nameDetails.get(n.name)} total={entry.lines} owner={name} />
                                </li>
                            ))}
                        </ul>
                    )}
                    <StrayNames strays={detail.strayNames ?? []} noise={entry.noise ?? 0} total={entry.lines} owner={name} />
                </section>
            </div>

            <div className="flex min-w-0 flex-col gap-4">
                <div className="flex items-center justify-between gap-3">
                    <h3 className="m-0 font-sans font-semibold text-[13px] text-foreground">{t("sprites.sheet.expressions", { count: flat.length })}</h3>
                    <Button variant="outline" size="sm" className="max-sm:min-h-11" onClick={() => void download()} disabled={busy || flat.length === 0}>
                        <DownloadIcon className="size-3.5" aria-hidden="true" />
                        {busy ? t("sprites.sheet.downloading") : t("sprites.sheet.download")}
                    </Button>
                </div>
                {failed ? <p className="m-0 font-sans text-[12px] text-destructive">{t("sprites.sheet.downloadFailed")}</p> : null}
                {scaled ? <p className="m-0 font-sans text-[11.5px] text-muted-foreground">{t("sprites.sheet.scaled")}</p> : null}
                {/* One roving tab stop: Tab enters the grid on the selected cell and the arrows move inside it. */}
                <div role="listbox" aria-label={t("sprites.sheet.gridAria")} onKeyDown={onKeyDown} className="flex flex-col gap-3">
                    {groups.map((g) => (
                        <div key={g.body} className="flex flex-col gap-1.5">
                            {groups.length > 1 ? <span className="font-mono text-[10.5px] text-muted-foreground">{t("sprites.sheet.body", { body: g.body })}</span> : null}
                            <div className="grid grid-cols-4 gap-1.5 sm:grid-cols-5 lg:grid-cols-6">
                                {g.variants.map((v, j) => {
                                    const i = g.start + j;
                                    const on = i === selected;
                                    return (
                                        <button
                                            key={v.key}
                                            ref={(el) => {
                                                cells.current[i] = el;
                                            }}
                                            type="button"
                                            role="option"
                                            aria-selected={on}
                                            tabIndex={on ? 0 : -1}
                                            aria-label={t("sprites.sheet.variantAria", { key: v.key })}
                                            onClick={() => setSelected(i)}
                                            className={cn("flex min-h-11 cursor-pointer flex-col overflow-hidden rounded-[9px] border bg-secondary/40 p-0 text-left ring-inset transition-colors focus-visible:ring-2 focus-visible:ring-ring/60", on ? "border-primary" : "border-border hover:border-primary/45")}
                                        >
                                            <SpriteFigure variant={v} name={entry.base} crop={CELL_CROP} lazy thumb={spriteThumbUrl(entry.base, v.key, server)} className="aspect-square w-full" />
                                            <span className={cn("truncate px-1 py-0.5 font-mono text-[10px] tabular-nums", on ? "text-primary" : "text-muted-foreground", v.uses === 0 && !on ? "opacity-60" : undefined)}>{v.key}</span>
                                        </button>
                                    );
                                })}
                            </div>
                        </div>
                    ))}
                </div>

                <section>
                    <h3 className="mt-1 mb-1.5 font-sans font-semibold text-[12px] text-muted-foreground uppercase tracking-[0.06em]">{t("sprites.sheet.stories")}</h3>
                    <ul className="m-0 flex max-h-72 list-none flex-col overflow-y-auto rounded-lg border border-border p-0">
                        {detail.stories.map((s) => (
                            <li key={s.id} className="border-border border-b last:border-b-0">
                                <Link to="/stories/$storyId" params={{ storyId: s.id }} className="flex min-h-11 items-center gap-3 px-3 py-1.5 no-underline transition-colors hover:bg-secondary/50 sm:min-h-9">
                                    <span className="flex min-w-0 flex-1 flex-col">
                                        <span className="truncate font-sans text-[12.5px] text-foreground">{s.name}</span>
                                        <span className="truncate font-sans text-[10.5px] text-muted-foreground">{s.tag ? `${s.groupName} · ${s.tag}` : s.groupName}</span>
                                    </span>
                                    <span className="shrink-0 font-mono text-[10.5px] text-muted-foreground tabular-nums">{s.lines > 0 ? t("sprites.sheet.lineCount", { count: lineCount(s.lines) }) : t("sprites.sheet.onStage")}</span>
                                </Link>
                            </li>
                        ))}
                    </ul>
                </section>
            </div>
        </div>
    );
}

/** Loading, the 404 and the sheet, for the dialog and the bottom sheet alike. */
function SpriteSheetState({ base }: { base: string }): React.ReactElement {
    const t: SpritesT = useT("story");
    const q = useSpriteDetail(base);
    if (q.isLoading) {
        return (
            <output className="grid gap-5 md:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]" aria-label={t("sprites.sheet.loading")}>
                <Skeleton className="aspect-square w-full rounded-xl" />
                <div className="grid grid-cols-4 gap-1.5 sm:grid-cols-6">
                    {Array.from({ length: SKELETON_CELLS }, (_, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: a placeholder has no identity beyond its position
                        <Skeleton key={i} className="aspect-square w-full rounded-[9px]" />
                    ))}
                </div>
            </output>
        );
    }
    if (q.isError) return <p className="py-8 text-center font-sans text-[13px] text-muted-foreground">{t("sprites.failed")}</p>;
    if (!q.data) return <p className="py-8 text-center font-sans text-[13px] text-muted-foreground">{t("sprites.sheet.notFound", { base })}</p>;
    return <SpriteSheetBody key={q.data.sprite.base} detail={q.data} />;
}

/** The title row: the primary name, then the folder id. */
function SheetTitleText({ name, base }: { name: string; base: string }): React.ReactElement {
    return (
        <span className="flex min-w-0 flex-col">
            <span className="truncate font-sans font-semibold text-[16px] text-foreground">{name}</span>
            <span className="truncate font-mono text-[10.5px] text-muted-foreground">{base}</span>
        </span>
    );
}

/**
 * The sheet OVER the grid: a Dialog above 640 px, a bottom Sheet under it.
 * The grid stays mounted underneath, so closing it returns to the same scroll
 * position; Escape closes it (the popup's own handler).
 */
export function SpriteSheetDialog({ base, name, onClose }: { base: string | null; name: string; onClose: () => void }): React.ReactElement | null {
    const t: SpritesT = useT("story");
    const phone = useMediaQuery("max-sm");
    if (!base) return null;
    const title = <SheetTitleText name={name} base={base} />;
    if (phone) {
        return (
            <Sheet open onOpenChange={(open) => !open && onClose()}>
                <SheetPopup side="bottom" className="max-h-[92dvh]" closeProps={{ className: "absolute end-2 top-2 max-sm:size-11" }}>
                    <SheetHeader>
                        <SheetTitle render={<div />}>{title}</SheetTitle>
                    </SheetHeader>
                    <SheetPanel className="overflow-y-auto">
                        <SpriteSheetState base={base} />
                    </SheetPanel>
                    <SheetFooter>
                        <SheetClose render={<Button variant="outline" className="min-h-11" />}>{t("sprites.sheet.close")}</SheetClose>
                    </SheetFooter>
                </SheetPopup>
            </Sheet>
        );
    }
    return (
        <Dialog open onOpenChange={(open) => !open && onClose()}>
            <DialogPopup className="max-w-5xl" closeProps={{ className: "absolute end-2 top-2" }}>
                <DialogHeader>
                    <DialogTitle render={<div />}>{title}</DialogTitle>
                </DialogHeader>
                <DialogPanel className="max-h-[78dvh] overflow-y-auto">
                    <SpriteSheetState base={base} />
                </DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button variant="outline" />}>{t("sprites.sheet.close")}</DialogClose>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}
