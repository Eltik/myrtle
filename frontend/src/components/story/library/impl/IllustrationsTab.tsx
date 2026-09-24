import type React from "react";
import { useMemo, useState } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages as browseMessages } from "./Browse.messages";
import type { LibGroup, LibRecord } from "./derive";
import { KindBadge } from "./GroupCard";
import { IllustrationPanel } from "./IllustrationPanel";
import type { messages } from "./IllustrationsTab.messages";
import { ART_KINDS, type ArtCategory, type ArtKind, artTabs, categoryTotal, countOf, type IArtGroup } from "./illustrations";
import type { messages as storiesMessages } from "./StoriesTab.messages";

/** The category labels are the Stories tab's own and the badge and chapter label the Browse cards', so all three catalogs answer here. */
type ArtT = TypedT<typeof messages & typeof storiesMessages & typeof browseMessages>;

export interface IIllustrationsTabProps {
    groups: LibGroup[];
    records: LibRecord[];
}

/**
 * The artwork the library knows about: two sub-tabs over the SAME cards, one
 * printing the illustration count and one the sprite count, filtered by the
 * Stories tab's categories with the operator records added at the end.
 *
 * A card opens a panel that fetches the group's artwork; nothing is fetched
 * until one is clicked, because the grid draws 451 cards and the payload for
 * one group is far larger than the whole index row that describes it.
 */
export function IllustrationsTab({ groups, records }: IIllustrationsTabProps): React.ReactElement {
    const t: ArtT = useT("story");
    const f = useFormatters();
    const tabs = useMemo(() => artTabs(groups, records), [groups, records]);
    const [kind, setKind] = useState<ArtKind>("illustrations");
    const [category, setCategory] = useState<ArtCategory | null>(null);
    const [open, setOpen] = useState<IArtGroup | null>(null);

    const active = useMemo(() => tabs.find((entry) => entry.key === category) ?? tabs[0], [tabs, category]);
    const total = useMemo(() => (active ? categoryTotal(active.groups, kind) : null), [active, kind]);

    return (
        <Tabs value={kind} onValueChange={(value) => setKind(String(value) as ArtKind)} className="gap-0">
            <TabsList variant="underline" className="msv-scroll w-full max-w-full justify-start overflow-x-auto border-border border-b pb-0">
                {ART_KINDS.map((key) => (
                    <TabsTab key={key} value={key} className="grow-0 px-3 max-sm:min-h-11">
                        {t(`illustrations.kind.${key}`)}
                    </TabsTab>
                ))}
            </TabsList>

            <div className="pt-4">
                <div className="mb-4 flex flex-wrap items-center gap-x-3 gap-y-2">
                    <fieldset className="msv-scroll flex min-w-0 max-w-full gap-0.5 overflow-x-auto rounded-[9px] border border-border bg-secondary/45 p-0.75" aria-label={t("illustrations.category.aria")}>
                        {tabs.map((entry) => (
                            <button
                                key={entry.key}
                                type="button"
                                onClick={() => setCategory(entry.key)}
                                aria-pressed={active?.key === entry.key}
                                className={cn("h-11 shrink-0 cursor-pointer rounded-md px-2.5 font-sans font-semibold text-[11.5px] transition-colors sm:h-6.5", active?.key === entry.key ? "bg-background text-foreground shadow-sm/5" : "text-muted-foreground hover:text-foreground")}
                            >
                                {entry.key === "record" ? t("illustrations.category.record") : t(`stories.tab.${entry.key}`)}
                                <span className="ms-1.5 font-mono text-[10px] text-muted-foreground tabular-nums">{entry.groups.length}</span>
                            </button>
                        ))}
                    </fieldset>
                    <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{total === null ? t("illustrations.totalUnknown") : t("illustrations.total", { count: f.number(total) })}</span>
                </div>

                {!active || active.groups.length === 0 ? (
                    <div className="rounded-[14px] border border-border border-dashed p-14 text-center font-sans text-[14px] text-muted-foreground">{t("illustrations.empty")}</div>
                ) : (
                    <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
                        {active.groups.map((group) => (
                            <ArtCard key={group.id} group={group} kind={kind} onOpen={() => setOpen(group)} />
                        ))}
                    </div>
                )}
            </div>

            <IllustrationPanel key={open?.id ?? "none"} group={open} kind={kind} onClose={() => setOpen(null)} />
        </Tabs>
    );
}

/**
 * One chapter's, or one operator's, artwork card: the chapter's KEY VISUAL, the
 * badge and chapter number the Browse cards print, the name, and the count for
 * the open sub-tab.
 *
 * The picture is `plateSource`, `bannerUrl` first and the derived `coverUrl`
 * second, which is the same choice the Browse tickets and the Reading Order
 * rows make; a record card keeps its operator AVATAR, which is the only picture
 * a record set has.
 */
function ArtCard({ group, kind, onOpen }: { group: IArtGroup; kind: ArtKind; onOpen: () => void }): React.ReactElement {
    const t: ArtT = useT("story");
    const f = useFormatters();
    const art = group.art.kind === "none" ? null : asset(group.art.url);
    const count = countOf(group, kind);
    const label = count === null ? t("illustrations.card.unknown") : f.number(count);

    return (
        <button
            type="button"
            onClick={onOpen}
            aria-label={t("illustrations.card.open", { name: group.name })}
            className="group flex cursor-pointer flex-col overflow-hidden rounded-xl border border-border bg-card text-left ring-inset transition-colors hover:border-primary/45 focus-visible:ring-2 focus-visible:ring-ring/60"
        >
            <span className="relative block aspect-16/9 w-full overflow-hidden bg-secondary/55">
                {art ? (
                    <img src={art} alt="" aria-hidden="true" loading="lazy" decoding="async" data-source={group.art.kind} className={cn("h-full w-full transition-transform duration-300 group-hover:scale-103", group.art.kind === "avatar" ? "object-contain" : "object-cover")} />
                ) : (
                    <span data-source="none" className="flex h-full w-full items-center justify-center bg-linear-to-br from-secondary/70 to-card px-2 text-center font-black font-heading text-[15px] text-muted-foreground/70 uppercase tracking-tight sm:text-[19px]">
                        {group.code}
                    </span>
                )}
                <KindBadge kind={group.kind} className="absolute top-1.5 left-1.5 shadow-sm/10" />
            </span>
            <span className="flex min-w-0 flex-1 flex-col gap-1 p-2.5">
                {group.chapter !== null ? <span className="font-mono text-[10px] text-muted-foreground tabular-nums">{t("browse.card.chapter", { n: group.chapter })}</span> : null}
                <span className="line-clamp-2 font-sans font-semibold text-[12.5px] text-foreground leading-snug sm:text-[13px]">{group.name}</span>
                <span className="mt-auto font-mono text-[10px] text-muted-foreground tabular-nums">{t(`illustrations.card.${kind}`, { count: label })}</span>
            </span>
        </button>
    );
}
