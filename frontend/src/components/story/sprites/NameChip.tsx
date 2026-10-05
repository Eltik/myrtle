import { Link } from "@tanstack/react-router";
import { ChevronDownIcon } from "lucide-react";
import type React from "react";
import { useState } from "react";
import { renderStoryNodes } from "#/components/story/reader/TextBox";
import { Button } from "#/components/ui/button";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { Sheet, SheetClose, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { useMediaQuery } from "#/hooks/use-media-query";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { renderLine } from "#/lib/story/text";
import { cn } from "#/lib/utils";
import type { StorySpriteNameDetail } from "#/types/generated/StorySpriteNameDetail";
import type { messages } from "./CharactersTab.messages";
import { lineCount } from "./gallery";

type SpritesT = TypedT<typeof messages>;

/** The nickname every example prints, as the reader does before a reader picks their own. */
const NICKNAME = "Doctor";

/** "Twin Swallow Pincer · 5 Before Operation": the story's name, then its node when it has one. */
export function storyNode(s: { name: string; code?: string; tag?: string }): string {
    const node = [s.code, s.tag].filter((p): p is string => typeof p === "string" && p.trim() !== "").join(" ");
    return node ? `${s.name} · ${node}` : s.name;
}

/** A name's share of the folder's named lines, 0..1, or null for an empty folder. */
export function nameShare(count: number, total: number): number | null {
    return total > 0 ? count / total : null;
}

/** What one name's popover shows: the count and share, the stories, the example lines. */
function NameDetailBody({ name, detail, total, owner }: { name: string; detail: StorySpriteNameDetail | undefined; total: number; owner: string }): React.ReactElement {
    const t: SpritesT = useT("story");
    const f = useFormatters();
    const count = detail?.count ?? 0;
    const share = nameShare(count, total);
    return (
        <div className="flex flex-col gap-3 text-left">
            <div className="flex flex-col gap-0.5">
                <span className="font-sans font-semibold text-[14px] text-foreground">{name}</span>
                <span className="font-mono text-[11px] text-muted-foreground tabular-nums">
                    {t("sprites.name.count", { count: lineCount(count) })}
                    {share !== null ? ` · ${t("sprites.name.share", { share: f.percent(share), owner })}` : ""}
                </span>
            </div>
            {detail && detail.stories.length > 0 ? (
                <section className="flex flex-col gap-1">
                    <span className="font-sans font-semibold text-[11px] text-muted-foreground uppercase tracking-[0.06em]">{t("sprites.name.stories")}</span>
                    <ul className="m-0 flex list-none flex-col p-0">
                        {detail.stories.map((s) => (
                            <li key={s.id} className="flex items-baseline justify-between gap-3 py-0.5 font-sans text-[12px]">
                                <Link to="/stories/$storyId" params={{ storyId: s.id }} className="min-w-0 truncate text-foreground underline-offset-2 hover:text-primary hover:underline max-sm:min-h-11 max-sm:leading-11">
                                    {storyNode(s)}
                                </Link>
                                <span className="shrink-0 font-mono text-[10.5px] text-muted-foreground tabular-nums">{t("sprites.sheet.lineCount", { count: lineCount(s.lines) })}</span>
                            </li>
                        ))}
                    </ul>
                    {detail.moreStories > 0 ? <span className="font-sans text-[11px] text-muted-foreground">{t("sprites.name.more", { count: detail.moreStories })}</span> : null}
                </section>
            ) : null}
            {detail && detail.examples.length > 0 ? (
                <section className="flex flex-col gap-1.5">
                    <span className="font-sans font-semibold text-[11px] text-muted-foreground uppercase tracking-[0.06em]">{t("sprites.name.examples")}</span>
                    {detail.examples.map((e) => (
                        <Link key={`${e.storyId}:${e.line}`} to="/stories/$storyId" params={{ storyId: e.storyId }} search={{ line: e.line }} className="group flex flex-col gap-0.5 rounded-md border border-border px-2.5 py-2 no-underline transition-colors hover:border-primary/45 max-sm:min-h-11">
                            <span className="whitespace-pre-line font-sans text-[12.5px] text-foreground leading-snug">{renderStoryNodes(renderLine(e.text, NICKNAME))}</span>
                            <span className="truncate font-sans text-[10.5px] text-muted-foreground group-hover:text-primary">{storyNode({ name: e.storyName, code: e.code, tag: e.tag })}</span>
                        </Link>
                    ))}
                </section>
            ) : null}
        </div>
    );
}

/**
 * One name under "Named in scripts": a chip that opens what the census knows
 * about it, a popover above 640 px and a bottom sheet under it. The data rides
 * the folder's sheet (`StorySpriteDetail.names`), so opening it fetches nothing.
 */
export function NameChip({ name, count, primary, stray = false, detail, total, owner }: { name: string; count: number; primary: boolean; stray?: boolean; detail: StorySpriteNameDetail | undefined; total: number; owner: string }): React.ReactElement {
    const t: SpritesT = useT("story");
    const f = useFormatters();
    const phone = useMediaQuery("max-sm");
    const [open, setOpen] = useState(false);
    const chip = cn(
        "flex cursor-pointer items-baseline gap-1.5 rounded-md border px-2 py-1 font-sans text-[12.5px] transition-colors hover:border-primary/60 focus-visible:outline-2 focus-visible:outline-ring max-sm:min-h-11 max-sm:items-center",
        primary ? "border-primary/40 bg-primary/10 text-foreground" : stray ? "border-border/70 border-dashed px-1.5 py-0.5 text-[11px] text-muted-foreground" : "border-border text-foreground/90",
    );
    const label = (
        <>
            <span>{name}</span>
            <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{f.number(lineCount(count))}</span>
        </>
    );
    if (phone) {
        return (
            <>
                <button type="button" className={chip} onClick={() => setOpen(true)} aria-haspopup="dialog" aria-label={t("sprites.name.open", { name })}>
                    {label}
                </button>
                <Sheet open={open} onOpenChange={setOpen}>
                    <SheetPopup side="bottom" className="max-h-[85dvh]" closeProps={{ className: "absolute end-2 top-2 size-11" }}>
                        <SheetHeader>
                            <SheetTitle className="sr-only">{name}</SheetTitle>
                        </SheetHeader>
                        <SheetPanel className="overflow-y-auto">
                            <NameDetailBody name={name} detail={detail} total={total} owner={owner} />
                        </SheetPanel>
                        <SheetFooter>
                            <SheetClose render={<Button variant="outline" className="min-h-11 w-full" />}>{t("sprites.sheet.close")}</SheetClose>
                        </SheetFooter>
                    </SheetPopup>
                </Sheet>
            </>
        );
    }
    return (
        <Popover open={open} onOpenChange={setOpen}>
            <PopoverTrigger
                render={(props) => (
                    <button {...props} type="button" className={chip} aria-label={t("sprites.name.open", { name })}>
                        {label}
                    </button>
                )}
            />
            <PopoverPopup side="bottom" align="start" className="w-[min(24rem,calc(100vw-2rem))]">
                <NameDetailBody name={name} detail={detail} total={total} owner={owner} />
            </PopoverPopup>
        </Popover>
    );
}

/** How many stray chips the open list mounts before "+K more". */
export const STRAY_CAP = 30;

/** The open list's chips: the first {@link STRAY_CAP}, or all of them once expanded. */
export function strayPage<T>(strays: readonly T[], all: boolean): { shown: readonly T[]; hidden: number } {
    if (all || strays.length <= STRAY_CAP) return { shown: strays, hidden: 0 };
    return { shown: strays.slice(0, STRAY_CAP), hidden: strays.length - STRAY_CAP };
}

/**
 * The names the alias threshold CUT, behind a disclosure: closed it reads
 * "N stray lines under M other names"; open it lists every one as a muted
 * chip, most lines first, each opening the same popover as an alias. They are
 * mostly someone else speaking while this sprite stays lit, which is why they
 * are listed here and never searched.
 */
export function StrayNames({ strays, noise, total, owner }: { strays: readonly StorySpriteNameDetail[]; noise: number; total: number; owner: string }): React.ReactElement | null {
    const t: SpritesT = useT("story");
    const [open, setOpen] = useState(false);
    const [all, setAll] = useState(false);
    const lines = lineCount(noise);
    if (strays.length === 0 || lines <= 0) return null;
    const { shown, hidden } = strayPage(strays, all);
    return (
        <div className="mt-1.5 flex flex-col gap-1.5">
            <button
                type="button"
                aria-expanded={open}
                onClick={() => setOpen((v) => !v)}
                className="flex w-fit cursor-pointer items-center gap-1 rounded-md px-1 py-0.5 font-sans text-[11px] text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring max-sm:min-h-11"
            >
                {t("sprites.stray.toggle", { lines, names: strays.length })}
                <ChevronDownIcon className={cn("size-3 transition-transform", open && "rotate-180")} aria-hidden="true" />
            </button>
            {open ? (
                <ul className="m-0 flex list-none flex-wrap gap-1 p-0">
                    {shown.map((n) => (
                        <li key={n.name}>
                            <NameChip name={n.name} count={n.count} primary={false} stray detail={n} total={total} owner={owner} />
                        </li>
                    ))}
                    {hidden > 0 ? (
                        <li>
                            <button type="button" onClick={() => setAll(true)} className="cursor-pointer rounded-md px-1.5 py-0.5 font-sans text-[11px] text-primary underline-offset-2 hover:underline focus-visible:outline-2 focus-visible:outline-ring max-sm:min-h-11">
                                {t("sprites.stray.more", { count: hidden })}
                            </button>
                        </li>
                    ) : null}
                </ul>
            ) : null}
        </div>
    );
}
