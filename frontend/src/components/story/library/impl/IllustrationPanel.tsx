import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import type React from "react";
import { useMemo, useState } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Sheet, SheetClose, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { Skeleton } from "#/components/ui/skeleton";
import { useMediaQuery } from "#/hooks/use-media-query";
import { storyIllustrationsQueryOptions } from "#/lib/api/story";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { IllustrationItem } from "#/types/generated/IllustrationItem";
import type { SpriteItem } from "#/types/generated/SpriteItem";
import type { messages } from "./IllustrationsTab.messages";
import { ART_PANES, type ArtKind, type ArtPane, artSummary, defaultPane, type IArtGroup, paneCounts, paneRows } from "./illustrations";

type ArtT = TypedT<typeof messages>;

export interface IIllustrationPanelProps {
    group: IArtGroup | null;
    kind: ArtKind;
    onClose: () => void;
}

/**
 * One group's artwork: a Dialog above 640 px, a bottom Sheet under it.
 *
 * `GET /story/group/{id}/illustrations` takes a story group id AND an operator
 * `charId` on the same route, so the record cards need no second query. A 404
 * is a FIRST-CLASS answer rather than a throw: the endpoint shipped after this
 * tab, so a backend that predates it gets a panel that says so and a page that
 * keeps working.
 */
export function IllustrationPanel({ group, kind, onClose }: IIllustrationPanelProps): React.ReactElement | null {
    const t: ArtT = useT("story");
    const phone = useMediaQuery("max-sm");
    const server = useGamedataServer();
    const groupId = group?.id ?? "";
    const query = useQuery({ ...storyIllustrationsQueryOptions(groupId, server), enabled: groupId !== "" });

    // The PANEL is where the split lives. A card prints `illustrationCount`,
    // the backgrounds and the CGs added together, because that is the only
    // figure the index carries; the three sub-tabs here are the split, and the
    // sub-tab the card was opened from picks which one lands first.
    const [pane, setPane] = useState<ArtPane>(() => defaultPane(kind));
    const rows = useMemo(() => paneRows(query.data, pane), [query.data, pane]);
    const counts = useMemo(() => paneCounts(query.data), [query.data]);
    const summary = useMemo(() => artSummary(rows), [rows]);

    const [viewer, setViewer] = useState<{ name: string; url: string } | null>(null);

    if (!group) return null;

    const plate = group.art.kind === "none" ? null : asset(group.art.url);
    const head = (
        <div className="flex min-w-0 items-center gap-2.5">
            {plate ? <img src={plate} alt="" aria-hidden="true" loading="lazy" decoding="async" data-source={group.art.kind} className={cn("aspect-16/9 w-16 shrink-0 rounded-[7px] bg-secondary/55", group.art.kind === "avatar" ? "object-contain" : "object-cover")} /> : null}
            <div className="flex min-w-0 flex-col">
                <span className="truncate font-sans font-semibold text-[15px] text-foreground">{group.name}</span>
                <span className="font-mono text-[10.5px] text-muted-foreground">{t(`illustrations.kind.${kind}`)}</span>
            </div>
        </div>
    );

    const panes = (
        <fieldset className="msv-scroll mb-2.5 flex max-w-full gap-0.5 overflow-x-auto rounded-[9px] border border-border bg-secondary/45 p-0.75" aria-label={t("illustrations.pane.aria")}>
            {ART_PANES.map((key) => (
                <button
                    key={key}
                    type="button"
                    onClick={() => setPane(key)}
                    aria-pressed={pane === key}
                    className={cn("h-11 shrink-0 cursor-pointer rounded-md px-2.5 font-sans font-semibold text-[11.5px] transition-colors sm:h-7", pane === key ? "bg-background text-foreground shadow-sm/5" : "text-muted-foreground hover:text-foreground")}
                >
                    {t(`illustrations.pane.${key}`)}
                    <span className="ms-1.5 font-mono text-[10px] text-muted-foreground tabular-nums">{counts[key]}</span>
                </button>
            ))}
        </fieldset>
    );

    const body = (
        <>
            {panes}
            <p className="mt-0 mb-2.5 font-sans text-[11px] text-muted-foreground/85 leading-relaxed">{t("illustrations.panel.split", { total: counts.backgrounds + counts.cgs, backgrounds: counts.backgrounds, cgs: counts.cgs })}</p>
            <Body loading={query.isLoading} failed={query.isError} unavailable={query.isSuccess && query.data === null} pane={pane} rows={rows} summary={summary} onView={setViewer} />
        </>
    );

    const viewerDialog = <ArtLightbox view={viewer} onClose={() => setViewer(null)} />;

    if (phone) {
        return (
            <>
                <Sheet open onOpenChange={(open) => !open && onClose()}>
                    <SheetPopup side="bottom" className="max-h-[88dvh]" closeProps={{ className: "absolute end-2 top-2 max-sm:size-11" }}>
                        <SheetHeader>
                            <SheetTitle render={<div />}>{head}</SheetTitle>
                        </SheetHeader>
                        <SheetPanel className="overflow-y-auto">{body}</SheetPanel>
                        <SheetFooter>
                            <SheetClose render={<Button variant="outline" className="min-h-11 w-full" />}>{t("illustrations.panel.close")}</SheetClose>
                        </SheetFooter>
                    </SheetPopup>
                </Sheet>
                {viewerDialog}
            </>
        );
    }

    return (
        <>
            <Dialog open onOpenChange={(open) => !open && onClose()}>
                <DialogPopup className="max-w-3xl" closeProps={{ className: "absolute end-2 top-2 max-sm:size-11" }}>
                    <DialogHeader>
                        <DialogTitle render={<div />}>{head}</DialogTitle>
                    </DialogHeader>
                    <DialogPanel className="max-h-[65dvh] overflow-y-auto">{body}</DialogPanel>
                    <DialogFooter>
                        <DialogClose render={<Button variant="outline" />}>{t("illustrations.panel.close")}</DialogClose>
                    </DialogFooter>
                </DialogPopup>
            </Dialog>
            {viewerDialog}
        </>
    );
}

export interface IArtView {
    name: string;
    url: string;
}

/**
 * ONE PIECE AT FULL SIZE, over whatever opened it.
 *
 * It is exported because the archive's gallery opens the same thing: two
 * lightboxes would be two sets of close-button wording, two max heights and
 * two chances for one of them to trap focus differently. `view` is nullable so
 * a caller renders it unconditionally and lets the null decide, which keeps
 * the mount and unmount in one place.
 */
export function ArtLightbox({ view, onClose }: { view: IArtView | null; onClose: () => void }): React.ReactElement | null {
    const t: ArtT = useT("story");
    if (!view) return null;
    return (
        <Dialog open onOpenChange={(open) => !open && onClose()}>
            <DialogPopup className="max-w-5xl" closeProps={{ className: "absolute end-2 top-2 max-sm:size-11" }}>
                <DialogHeader>
                    <DialogTitle className="truncate font-mono text-[12.5px]">{view.name}</DialogTitle>
                </DialogHeader>
                <DialogPanel className="flex max-h-[72dvh] items-center justify-center overflow-auto bg-secondary/35">
                    <img src={view.url} alt={view.name} decoding="async" className="max-h-[70dvh] w-auto max-w-full object-contain" />
                </DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button variant="outline" className="max-sm:min-h-11" />}>{t("illustrations.viewer.close")}</DialogClose>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}

/** Loading, the 404, the failure, the empty case and the grid, so all three sub-tabs read the same. */
function Body({
    loading,
    failed,
    unavailable,
    pane,
    rows,
    summary,
    onView,
}: {
    loading: boolean;
    failed: boolean;
    unavailable: boolean;
    pane: ArtPane;
    rows: (IllustrationItem | SpriteItem)[];
    summary: { rows: number; missing: number; stories: number };
    onView: (viewer: { name: string; url: string }) => void;
}): React.ReactElement {
    const t: ArtT = useT("story");
    const f = useFormatters();

    if (unavailable) {
        return (
            <div className="rounded-[12px] border border-border border-dashed p-8 text-center">
                <p className="m-0 font-sans text-[13px] text-muted-foreground">{t("illustrations.panel.unavailable")}</p>
                <p className="mt-1.5 mb-0 font-mono text-[11px] text-muted-foreground/80">{t("illustrations.panel.unavailableNote")}</p>
            </div>
        );
    }
    if (failed) return <p className="py-8 text-center font-sans text-[13px] text-muted-foreground">{t("illustrations.panel.failed")}</p>;
    if (loading) {
        return (
            <output className="grid grid-cols-2 gap-2.5 sm:grid-cols-3" aria-label={t("illustrations.panel.loading")}>
                {[0, 1, 2, 3, 4, 5].map((i) => (
                    <Skeleton key={i} className="aspect-16/9 w-full rounded-[10px]" />
                ))}
            </output>
        );
    }
    if (summary.rows === 0) return <div className="rounded-[12px] border border-border border-dashed p-8 text-center font-sans text-[13px] text-muted-foreground">{t("illustrations.panel.empty")}</div>;

    return (
        <>
            <p className="mt-0 mb-2.5 flex flex-wrap gap-x-3 font-mono text-[10.5px] text-muted-foreground tabular-nums">
                <span>{t("illustrations.panel.summary", { rows: f.number(summary.rows), stories: summary.stories })}</span>
                {summary.missing > 0 ? <span>{t("illustrations.panel.missing", { count: f.number(summary.missing) })}</span> : null}
            </p>
            <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-3">{pane === "sprites" ? (rows as SpriteItem[]).map((sprite) => <SpriteTile key={sprite.base} sprite={sprite} onView={onView} />) : (rows as IllustrationItem[]).map((item) => <IllustrationTile key={item.name} item={item} onView={onView} />)}</div>
        </>
    );
}

/** One background or CG: a 16:9 thumbnail that opens the full-size viewer, and the stories that use it. */
function IllustrationTile({ item, onView }: { item: IllustrationItem; onView: (viewer: { name: string; url: string }) => void }): React.ReactElement {
    const url = item.url ? asset(item.url) : null;
    return <Tile name={item.name} url={url} aspect="aspect-16/9" storyIds={item.storyIds} onView={onView} />;
}

/** One sprite base: the `$1` body at its own square aspect, with the face count the group asks for. */
function SpriteTile({ sprite, onView }: { sprite: SpriteItem; onView: (viewer: { name: string; url: string }) => void }): React.ReactElement {
    const t: ArtT = useT("story");
    const url = sprite.bodyUrl ? asset(sprite.bodyUrl) : null;
    return <Tile name={sprite.base} url={url} aspect="aspect-square" storyIds={sprite.storyIds} onView={onView} extra={t("illustrations.item.faces", { count: sprite.faces })} />;
}

/**
 * The shared tile. The thumbnail is LAZY and sits in a fixed-aspect box, so a
 * group with a hundred pieces lays out in one pass and only fetches what
 * scrolls into view.
 */
function Tile({ name, url, aspect, storyIds, extra, onView }: { name: string; url: string | null; aspect: string; storyIds: readonly string[]; extra?: string; onView: (viewer: { name: string; url: string }) => void }): React.ReactElement {
    const t: ArtT = useT("story");
    const first = storyIds[0];

    return (
        <div className="flex min-w-0 flex-col gap-1">
            {url ? (
                <button
                    type="button"
                    onClick={() => onView({ name, url })}
                    aria-label={t("illustrations.item.view", { name })}
                    className={cn(aspect, "block w-full cursor-pointer overflow-hidden rounded-[10px] border border-border bg-secondary/55 ring-inset transition-colors hover:border-primary/45 focus-visible:ring-2 focus-visible:ring-ring/60")}
                >
                    <img src={url} alt="" aria-hidden="true" loading="lazy" decoding="async" className={cn("h-full w-full", aspect === "aspect-square" ? "object-contain" : "object-cover")} />
                </button>
            ) : (
                <div className={cn(aspect, "flex w-full items-center justify-center rounded-[10px] border border-border border-dashed bg-secondary/25 px-2 text-center font-sans text-[10.5px] text-muted-foreground/70")}>{t("illustrations.item.notExtracted")}</div>
            )}
            <span className="truncate font-mono text-[10.5px] text-foreground" title={name}>
                {name}
            </span>
            <span className="flex flex-wrap items-center gap-x-2 font-mono text-[10px] text-muted-foreground tabular-nums">
                {extra ? <span>{extra}</span> : null}
                {first ? (
                    <Link to="/stories/$storyId" params={{ storyId: first }} className="text-muted-foreground underline-offset-2 transition-colors hover:text-primary hover:underline max-sm:min-h-11 max-sm:leading-11">
                        {t("illustrations.item.usedIn", { count: storyIds.length })}
                    </Link>
                ) : (
                    <span>{t("illustrations.item.usedIn", { count: storyIds.length })}</span>
                )}
            </span>
        </div>
    );
}
