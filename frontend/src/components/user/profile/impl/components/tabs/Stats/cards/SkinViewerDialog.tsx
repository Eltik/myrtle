import { useQuery } from "@tanstack/react-query";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Check, Search, X } from "lucide-react";
import { memo, useCallback, useDeferredValue, useEffect, useMemo, useRef, useState, useTransition } from "react";
import { DetailRow, type ISkinPrice, SkinDetailContent } from "#/components/skins/SkinDetailDialog";
import { Dialog, DialogContent, DialogTitle } from "#/components/ui/dialog";
import { ScrollArea } from "#/components/ui/scroll-area";
import { type ISkinIndexEntry, skinPopularityQueryOptions } from "#/lib/api/skins";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, getAvatarById } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import type { messages } from "./SkinViewerDialog.messages";

type ViewerT = TypedT<typeof messages>;

/** A key in `SkinViewerDialog.messages.ts`; resolved by whichever part renders it. */
type MessageKey = keyof typeof messages & string;

type OwnershipFilter = "all" | "missing" | "owned";
type SortMode = "brand" | "date" | "popularity";

interface ISkinPopularityInfo {
    /** Fraction in [0, 1] of users that own this skin, or null if unknown. */
    pct: number | null;
    owners: number;
}

interface ISkinViewerDialogProps {
    skins: ISkinIndexEntry[];
    ownedIds: Set<string>;
    /** Authoritative owned count from the user profile. Used for the header display
     *  so it's correct even before the per-skin ownership list finishes loading. */
    profileOwnedCount: number;
    operatorsMap: Map<string, IOperatorListItem>;
    color: string;
}

const FILTER_TABS: { id: OwnershipFilter; labelKey: MessageKey }[] = [
    { id: "all", labelKey: "profile.skins.filter.all" },
    { id: "owned", labelKey: "profile.skins.filter.owned" },
    { id: "missing", labelKey: "profile.skins.filter.missing" },
];

const SORT_TABS: { id: SortMode; labelKey: MessageKey }[] = [
    { id: "brand", labelKey: "profile.skins.sort.brand" },
    { id: "date", labelKey: "profile.skins.sort.date" },
    { id: "popularity", labelKey: "profile.skins.sort.popularity" },
];

const SECTION_STYLE: React.CSSProperties = {
    contentVisibility: "auto",
    containIntrinsicSize: "600px",
};

const CARD_STYLE: React.CSSProperties = { contain: "content" };

const INITIAL_RENDER_CHUNK = 150;
const RENDER_CHUNK_STEP = 150;

const COL_BREAKPOINTS: readonly { minWidth: number; cols: number }[] = [
    { minWidth: 1536, cols: 12 },
    { minWidth: 1280, cols: 10 },
    { minWidth: 1024, cols: 8 },
    { minWidth: 768, cols: 6 },
    { minWidth: 640, cols: 4 },
    { minWidth: 0, cols: 3 },
];

const VIRTUAL_SCROLL_MARGIN = 12;
const VIRTUAL_ROW_ESTIMATE_PX = 160;
const VIRTUAL_ROW_OVERSCAN = 4;

interface ICardData {
    skin: ISkinIndexEntry;
    op: IOperatorListItem | undefined;
    opName: string;
    skinName: string;
    searchable: string;
    avatarURL: string;
    price: ISkinPrice;
}

export function SkinViewerDialog(props: ISkinViewerDialogProps) {
    const t: ViewerT = useT("user");
    const [query, setQuery] = useState("");
    const [filter, setFilter] = useState<OwnershipFilter>("missing");
    const [sort, setSort] = useState<SortMode>("brand");

    return (
        <DialogContent bottomStickOnMobile={false} className="flex h-[95vh] max-h-[95vh] w-[95vw] max-w-[95vw] flex-col overflow-hidden p-0 sm:max-w-[95vw]" showCloseButton>
            <DialogTitle className="sr-only">{t("profile.skins.title")}</DialogTitle>
            <SkinViewerBody {...props} filter={filter} query={query} setFilter={setFilter} setQuery={setQuery} setSort={setSort} sort={sort} />
        </DialogContent>
    );
}

interface ISkinViewerBodyProps extends ISkinViewerDialogProps {
    query: string;
    setQuery: (q: string) => void;
    filter: OwnershipFilter;
    setFilter: (f: OwnershipFilter) => void;
    sort: SortMode;
    setSort: (s: SortMode) => void;
}

function SkinViewerBody({ skins, ownedIds, profileOwnedCount, operatorsMap, color, query, setQuery, filter, setFilter, sort, setSort }: ISkinViewerBodyProps) {
    const t: ViewerT = useT("user");
    const f = useFormatters();
    const deferredQuery = useDeferredValue(query);
    const deferredFilter = useDeferredValue(filter);
    const deferredSort = useDeferredValue(sort);
    const [selectedSkinId, setSelectedSkinId] = useState<string | null>(null);
    const [renderBudget, setRenderBudget] = useState(INITIAL_RENDER_CHUNK);
    const [, startRenderTransition] = useTransition();

    const { data: popularity } = useQuery(skinPopularityQueryOptions());

    const popularityMap = useMemo(() => {
        if (!popularity || popularity.totalUsers <= 0) return null;
        const total = popularity.totalUsers;
        const map = new Map<string, ISkinPopularityInfo>();
        for (const [skinId, owners] of Object.entries(popularity.counts)) {
            map.set(skinId, { owners, pct: owners / total });
        }
        return map;
    }, [popularity]);

    const cards = useMemo<ICardData[]>(() => {
        const out: ICardData[] = [];
        for (const s of skins) {
            if (!s.skinId?.includes("@")) continue;
            const op = operatorsMap.get(s.charId);
            const opName = op?.name ?? "";
            const rawSkinName = s.displaySkin?.skinName ?? s.displaySkin?.skinGroupName ?? t("profile.skins.card.fallbackName");
            const groupName = s.displaySkin?.skinGroupName ?? "";
            const skinNameForSearch = (s.displaySkin?.skinName ?? "").toLowerCase();
            out.push({
                skin: s,
                op,
                opName,
                skinName: rawSkinName,
                searchable: `${opName.toLowerCase()} ${skinNameForSearch} ${groupName.toLowerCase()}`,
                avatarURL: getAvatarById(s.skinId),
                price: getSkinPrice(s, t),
            });
        }
        return out;
    }, [skins, operatorsMap, t]);

    const enumeratedOwnedCount = useMemo(() => {
        if (profileOwnedCount > 0) return 0;
        let n = 0;
        for (const c of cards) if (ownedIds.has(c.skin.skinId)) n++;
        return n;
    }, [cards, ownedIds, profileOwnedCount]);
    const ownedCount = profileOwnedCount > 0 ? profileOwnedCount : enumeratedOwnedCount;

    const totalCount = cards.length;
    const missingCount = Math.max(0, totalCount - ownedCount);

    const filteredCards = useMemo(() => {
        const q = deferredQuery.trim().toLowerCase();
        return cards.filter((c) => {
            const owned = ownedIds.has(c.skin.skinId);
            if (deferredFilter === "owned" && !owned) return false;
            if (deferredFilter === "missing" && owned) return false;
            if (!q) return true;
            return c.searchable.includes(q);
        });
    }, [cards, deferredQuery, deferredFilter, ownedIds]);

    const sections = useMemo(() => buildSections(filteredCards, deferredSort, popularityMap, t), [filteredCards, deferredSort, popularityMap, t]);
    const totalFiltered = filteredCards.length;

    useEffect(() => {
        if (deferredSort !== "brand") return;
        if (renderBudget >= filteredCards.length) return;
        const id = requestAnimationFrame(() => {
            startRenderTransition(() => {
                setRenderBudget((c) => Math.min(c + RENDER_CHUNK_STEP, filteredCards.length));
            });
        });
        return () => cancelAnimationFrame(id);
    }, [renderBudget, filteredCards.length, deferredSort]);

    const renderedSections = useMemo(() => {
        if (renderBudget >= filteredCards.length) return sections;
        const out: ISkinSection[] = [];
        let remaining = renderBudget;
        for (const section of sections) {
            if (remaining <= 0) break;
            if (section.cards.length <= remaining) {
                out.push(section);
                remaining -= section.cards.length;
            } else {
                out.push({ ...section, cards: section.cards.slice(0, remaining) });
                remaining = 0;
            }
        }
        return out;
    }, [sections, renderBudget, filteredCards.length]);

    const handleSelect = useCallback((skinId: string) => setSelectedSkinId(skinId), []);
    const handleDetailOpenChange = useCallback((open: boolean) => {
        if (!open) setSelectedSkinId(null);
    }, []);
    const clearSearch = useCallback(() => setQuery(""), [setQuery]);
    const handleQueryChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => setQuery(e.target.value), [setQuery]);

    const selectedCard = useMemo(() => {
        if (!selectedSkinId) return null;
        for (const c of cards) if (c.skin.skinId === selectedSkinId) return c;
        return null;
    }, [selectedSkinId, cards]);

    return (
        <>
            <header className="flex shrink-0 flex-col gap-3 border-border/60 border-b bg-card/60 p-4 backdrop-blur sm:p-5">
                <div className="flex flex-wrap items-baseline gap-x-4 gap-y-1">
                    <h2 className="font-heading font-semibold text-lg leading-none sm:text-xl">{t("profile.skins.title")}</h2>
                    <p className="font-mono text-[11px] text-muted-foreground tabular-nums">
                        <span className="font-semibold" style={{ color }}>
                            {f.number(missingCount)}
                        </span>{" "}
                        {t("profile.skins.header.missing")} · <span className="font-semibold text-foreground">{f.number(ownedCount)}</span> {t("profile.skins.header.owned")} · {f.number(totalCount)} {t("profile.skins.header.total")}
                    </p>
                </div>

                <div className="flex flex-col gap-2 sm:flex-row sm:flex-wrap sm:items-center sm:gap-3">
                    <FilterTabs color={color} counts={{ missing: missingCount, owned: ownedCount, all: totalCount }} onChange={setFilter} value={filter} />
                    <SortTabs onChange={setSort} value={sort} />
                    <div className="relative flex items-center sm:ml-auto">
                        <Search aria-hidden className="absolute left-2.5 h-3.5 w-3.5 text-muted-foreground/60" />
                        <input
                            aria-label={t("profile.skins.search.aria")}
                            className="w-full rounded-md border border-border bg-background py-1.5 pr-8 pl-8 text-sm outline-none transition-colors placeholder:text-muted-foreground/50 focus:border-foreground/30 sm:w-64"
                            onChange={handleQueryChange}
                            placeholder={t("profile.skins.search.placeholder")}
                            type="search"
                            value={query}
                        />
                        {query && (
                            <button aria-label={t("profile.skins.search.clear")} className="absolute right-1.5 flex h-6 w-6 cursor-pointer items-center justify-center rounded-full text-muted-foreground hover:bg-muted hover:text-foreground" onClick={clearSearch} type="button">
                                <X className="h-3 w-3" />
                            </button>
                        )}
                    </div>
                </div>
            </header>

            {totalFiltered === 0 ? (
                <div className="flex flex-1 flex-col items-center justify-center gap-2 p-8 text-center">
                    <p className="font-medium text-sm">{t("profile.skins.empty.title")}</p>
                    <p className="text-muted-foreground text-xs">{deferredQuery ? t("profile.skins.empty.withQuery") : t("profile.skins.empty.noQuery")}</p>
                </div>
            ) : (
                <ScrollArea className="min-h-0 flex-1">
                    {deferredSort === "brand" ? (
                        <div className="flex flex-col gap-5 px-3 pt-3 pb-4 sm:gap-6 sm:px-4 sm:pb-5">
                            {renderedSections.map((section) => (
                                <section className="flex flex-col gap-2" key={section.key} style={SECTION_STYLE}>
                                    <SectionHeader color={color} count={section.cards.length} tag={section.tag} title={section.title} />
                                    <div className="grid grid-cols-3 gap-2 sm:grid-cols-4 sm:gap-2.5 md:grid-cols-6 lg:grid-cols-8 xl:grid-cols-10 2xl:grid-cols-12">
                                        {section.cards.map((c) => (
                                            <SkinCard card={c} color={color} key={c.skin.skinId} onSelect={handleSelect} owned={ownedIds.has(c.skin.skinId)} popularity={popularityMap?.get(c.skin.skinId) ?? null} />
                                        ))}
                                    </div>
                                </section>
                            ))}
                        </div>
                    ) : (
                        <div className="px-3 pt-3 pb-4 sm:px-4 sm:pb-5">
                            <VirtualizedSkinGrid cards={sections[0]?.cards ?? []} color={color} onSelect={handleSelect} ownedIds={ownedIds} popularityMap={popularityMap} />
                        </div>
                    )}
                </ScrollArea>
            )}

            <Dialog onOpenChange={handleDetailOpenChange} open={selectedCard !== null}>
                {selectedCard && <SkinDetailDialog card={selectedCard} color={color} owned={ownedIds.has(selectedCard.skin.skinId)} popularity={popularityMap?.get(selectedCard.skin.skinId) ?? null} />}
            </Dialog>
        </>
    );
}

interface IFilterTabsProps {
    value: OwnershipFilter;
    onChange: (v: OwnershipFilter) => void;
    counts: Record<OwnershipFilter, number>;
    color: string;
}

function FilterTabs({ value, onChange, counts, color }: IFilterTabsProps) {
    const t: ViewerT = useT("user");
    const f = useFormatters();
    return (
        <div className="inline-flex shrink-0 items-center gap-0.5 rounded-md border border-border bg-muted/40 p-0.5">
            {FILTER_TABS.map((tab) => {
                const active = value === tab.id;
                return (
                    <button
                        aria-pressed={active}
                        className={cn("flex cursor-pointer items-center gap-1.5 rounded-[5px] px-2.5 py-1 font-mono text-[11px] uppercase tracking-wider transition-colors", active ? "bg-background text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground")}
                        key={tab.id}
                        onClick={() => onChange(tab.id)}
                        style={active && tab.id === "missing" ? { color } : undefined}
                        type="button"
                    >
                        {t(tab.labelKey)}
                        <span className="font-semibold tabular-nums">{f.number(counts[tab.id])}</span>
                    </button>
                );
            })}
        </div>
    );
}

interface ISortTabsProps {
    value: SortMode;
    onChange: (v: SortMode) => void;
}

function SortTabs({ value, onChange }: ISortTabsProps) {
    const t: ViewerT = useT("user");
    return (
        <div className="inline-flex shrink-0 items-center gap-0.5 rounded-md border border-border bg-muted/40 p-0.5">
            <span className="px-1.5 font-mono text-[10px] text-muted-foreground uppercase tracking-wider">{t("profile.skins.sort.label")}</span>
            {SORT_TABS.map((tab) => {
                const active = value === tab.id;
                return (
                    <button
                        aria-pressed={active}
                        className={cn("cursor-pointer rounded-[5px] px-2 py-1 font-mono text-[11px] uppercase tracking-wider transition-colors", active ? "bg-background text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground")}
                        key={tab.id}
                        onClick={() => onChange(tab.id)}
                        type="button"
                    >
                        {t(tab.labelKey)}
                    </button>
                );
            })}
        </div>
    );
}

interface ISkinCardProps {
    card: ICardData;
    owned: boolean;
    color: string;
    popularity: ISkinPopularityInfo | null;
    onSelect: (skinId: string) => void;
}

const SkinCard = memo(function SkinCard({ card, owned, color, popularity, onSelect }: ISkinCardProps) {
    const t: ViewerT = useT("user");
    const { skin, opName, skinName, price, avatarURL } = card;
    const displayOpName = opName || skin.charId;
    const handleClick = () => onSelect(skin.skinId);

    return (
        <button
            aria-label={t("profile.skins.card.aria", { operator: displayOpName, skin: skinName })}
            className={cn("group relative flex cursor-pointer flex-col overflow-hidden rounded-lg border bg-card text-left transition-all", "hover:-translate-y-0.5 hover:border-foreground/30 hover:shadow-md", owned ? "border-border/60" : "border-border")}
            onClick={handleClick}
            style={CARD_STYLE}
            type="button"
        >
            <div className="relative aspect-square w-full overflow-hidden bg-muted/30">
                <img alt="" aria-hidden className={cn("h-full w-full object-cover object-center transition-transform duration-300 group-hover:scale-[1.04]", !owned && "opacity-60 saturate-50 group-hover:opacity-85 group-hover:saturate-75")} decoding="async" loading="lazy" src={avatarURL} />
                {!owned && <span aria-hidden className="pointer-events-none absolute inset-0 bg-linear-to-t from-background/40 to-transparent" />}
                {price.label && (
                    <span className="absolute top-1 left-1">
                        <PriceChip price={price} />
                    </span>
                )}
                <span className="absolute top-1 right-1">
                    <OwnershipBadge color={color} owned={owned} />
                </span>
                {popularity && popularity.pct !== null && (
                    <span className="absolute right-1 bottom-1">
                        <PopularityChip color={color} info={popularity} />
                    </span>
                )}
            </div>
            <div className="flex min-w-0 flex-col gap-0.5 px-1.5 py-1.5">
                <span className="truncate font-medium text-[11px] leading-tight">{skinName}</span>
                <span className="truncate text-[10px] text-muted-foreground leading-tight">{displayOpName}</span>
            </div>
        </button>
    );
});

const PopularityChip = memo(function PopularityChip({ info, color }: { info: ISkinPopularityInfo; color: string }) {
    const t: ViewerT = useT("user");
    const f = useFormatters();
    const pct = info.pct ?? 0;
    const label = f.percent(pct);
    return (
        <span className="flex items-center gap-0.5 rounded-full bg-background/85 px-1.5 py-px font-mono font-semibold text-[9px] uppercase tabular-nums tracking-wider shadow-sm" style={{ color }} title={t("profile.skins.popularity.tooltip", { owners: f.number(info.owners), pct: (pct * 100).toFixed(2) })}>
            {label}
        </span>
    );
});

const PriceChip = memo(function PriceChip({ price }: { price: ISkinPrice }) {
    const isFree = price.kind === "free";
    return (
        <span className={cn("flex items-center gap-0.5 rounded-full px-1.5 py-px font-mono font-semibold text-[9px] uppercase tracking-wider shadow-sm", isFree ? "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400" : "bg-background/85 text-foreground")} title={price.tooltip ?? undefined}>
            {price.label}
        </span>
    );
});

const OwnershipBadge = memo(function OwnershipBadge({ owned, color }: { owned: boolean; color: string }) {
    const t: ViewerT = useT("user");
    if (owned) {
        return (
            <span aria-label={t("profile.skins.badge.owned")} className="flex h-5 w-5 items-center justify-center rounded-full bg-emerald-500/95 text-white shadow-sm" role="img">
                <Check aria-hidden className="h-3 w-3" strokeWidth={3} />
            </span>
        );
    }
    return <span aria-label={t("profile.skins.badge.missing")} className="block h-5 w-5 rounded-full border bg-background/85" role="img" style={{ borderColor: `color-mix(in oklch, ${color} 50%, transparent)` }} />;
});

function getColumnCount(): number {
    if (typeof window === "undefined") return COL_BREAKPOINTS[0].cols;
    const w = window.innerWidth;
    for (const { minWidth, cols } of COL_BREAKPOINTS) {
        if (w >= minWidth) return cols;
    }
    return COL_BREAKPOINTS[COL_BREAKPOINTS.length - 1].cols;
}

function useColumnCount(): number {
    const [cols, setCols] = useState<number>(getColumnCount);
    useEffect(() => {
        const handler = () => setCols(getColumnCount());
        handler();
        window.addEventListener("resize", handler, { passive: true });
        return () => window.removeEventListener("resize", handler);
    }, []);
    return cols;
}

interface IVirtualizedSkinGridProps {
    cards: ICardData[];
    color: string;
    ownedIds: Set<string>;
    popularityMap: Map<string, ISkinPopularityInfo> | null;
    onSelect: (skinId: string) => void;
}

const CARD_TEXT_HEIGHT_PX = 32;

function gapForCols(cols: number): number {
    return cols <= 3 ? 8 : 10;
}

function VirtualizedSkinGrid({ cards, color, ownedIds, popularityMap, onSelect }: IVirtualizedSkinGridProps) {
    const cols = useColumnCount();

    const rows = useMemo(() => {
        const out: ICardData[][] = [];
        for (let i = 0; i < cards.length; i += cols) {
            out.push(cards.slice(i, i + cols));
        }
        return out;
    }, [cards, cols]);

    const innerRef = useRef<HTMLDivElement | null>(null);
    const [scrollEl, setScrollEl] = useState<HTMLElement | null>(null);
    const [containerWidth, setContainerWidth] = useState(0);

    useEffect(() => {
        const el = innerRef.current;
        if (!el) return;
        let cur: HTMLElement | null = el.parentElement;
        while (cur) {
            if (cur.getAttribute("data-slot") === "scroll-area-viewport") {
                setScrollEl(cur);
                break;
            }
            cur = cur.parentElement;
        }
        const observer = new ResizeObserver((entries) => {
            const w = entries[0]?.contentRect.width ?? 0;
            if (w > 0) setContainerWidth(w);
        });
        observer.observe(el);
        return () => observer.disconnect();
    }, []);

    const rowHeight = useMemo(() => {
        if (containerWidth <= 0) return VIRTUAL_ROW_ESTIMATE_PX;
        const gap = gapForCols(cols);
        const cardWidth = (containerWidth - (cols - 1) * gap) / cols;
        return Math.round(cardWidth + CARD_TEXT_HEIGHT_PX + gap);
    }, [containerWidth, cols]);

    const virtualizer = useVirtualizer({
        count: rows.length,
        getScrollElement: () => scrollEl,
        estimateSize: () => rowHeight,
        overscan: VIRTUAL_ROW_OVERSCAN,
        scrollMargin: VIRTUAL_SCROLL_MARGIN,
    });

    useEffect(() => {
        virtualizer.measure();
    }, [virtualizer]);

    const virtualItems = virtualizer.getVirtualItems();
    const totalSize = virtualizer.getTotalSize();

    return (
        <div ref={innerRef} style={{ height: totalSize, position: "relative", width: "100%" }}>
            {virtualItems.map((vi) => {
                const row = rows[vi.index];
                if (!row) return null;
                return (
                    <div
                        key={vi.key}
                        style={{
                            contain: "content",
                            height: rowHeight,
                            left: 0,
                            position: "absolute",
                            top: 0,
                            transform: `translateY(${vi.start}px)`,
                            width: "100%",
                            willChange: "transform",
                        }}
                    >
                        <div className="grid grid-cols-3 gap-2 sm:grid-cols-4 sm:gap-2.5 md:grid-cols-6 lg:grid-cols-8 xl:grid-cols-10 2xl:grid-cols-12">
                            {row.map((c) => (
                                <SkinCard card={c} color={color} key={c.skin.skinId} onSelect={onSelect} owned={ownedIds.has(c.skin.skinId)} popularity={popularityMap?.get(c.skin.skinId) ?? null} />
                            ))}
                        </div>
                    </div>
                );
            })}
        </div>
    );
}

interface ISkinDetailDialogProps {
    card: ICardData;
    owned: boolean;
    color: string;
    popularity: ISkinPopularityInfo | null;
}

function SkinDetailDialog({ card, owned, color, popularity }: ISkinDetailDialogProps) {
    const t: ViewerT = useT("user");
    const f = useFormatters();
    const { skin, skinName, price, avatarURL } = card;
    const opName = card.opName || skin.charId;
    return (
        <SkinDetailContent
            skin={skin}
            opName={opName}
            skinName={skinName}
            avatarURL={avatarURL}
            price={price}
            corner={<OwnershipBadge color={color} owned={owned} />}
            closeLabel={t("profile.skins.detail.close")}
            extraRows={
                popularity && popularity.pct !== null ? (
                    <DetailRow label={t("profile.skins.detail.popularity")}>
                        <span className="font-semibold" style={{ color }}>
                            {f.percent(popularity.pct)}
                        </span>
                        <span className="ml-2 text-muted-foreground text-xs">
                            {t("profile.skins.detail.ofUsers")} · {t("profile.skins.detail.ownerCount", { n: f.number(popularity.owners) })}
                        </span>
                    </DetailRow>
                ) : null
            }
        />
    );
}

// ─── Sectioning by brand + acquisition channel ─────────────────────────────

/** Visual tag shown on a section header to indicate how the brand is acquired.
 *  Maps the underlying `displayTagId` (or absence of one) into a short label. */
type SectionChannel = "collab" | "event" | "is" | "seasonal" | "special-pack" | "code-exchange" | "store";

interface ISkinSection {
    key: string;
    title: string;
    channel: SectionChannel;
    tag: string | null;
    sortIndex: number;
    cards: ICardData[];
}

const CHANNEL_PRIORITY: Record<SectionChannel, number> = {
    // Lower number = appears earlier in the dialog.
    collab: 0,
    store: 1,
    seasonal: 2,
    "special-pack": 3,
    is: 4,
    event: 5,
    "code-exchange": 6,
};

const CHANNEL_LABEL: Record<SectionChannel, MessageKey | null> = {
    collab: "profile.skins.channel.collab",
    store: null, // implicit - main paid brands are the baseline, no chip needed
    seasonal: "profile.skins.channel.seasonal",
    "special-pack": "profile.skins.channel.specialPack",
    is: "profile.skins.channel.is",
    event: "profile.skins.channel.event",
    "code-exchange": "profile.skins.channel.codeExchange",
};

/** The channel chip for a section, or null where no chip is drawn. */
function channelLabel(channel: SectionChannel, t: ViewerT): string | null {
    const key = CHANNEL_LABEL[channel];
    return key ? t(key) : null;
}

function classifyChannel(tagId: string | null | undefined): SectionChannel {
    if (!tagId) return "store";
    if (tagId === "From collabs") return "collab";
    if (tagId === "Event Reward") return "event";
    if (tagId === "Integrated Strategies") return "is";
    if (tagId === "Seasonal Attire") return "seasonal";
    if (tagId === "Obtain from Special Pack") return "special-pack";
    if (tagId === "Official Artworks") return "code-exchange";
    return "store";
}

// Strip the iteration suffix from a skin group name so all iterations of the
// same brand fold into a single section. "Witch Feast/V" → "Witch Feast",
// "Coral Coast/XX" → "Coral Coast", but a one-word brand like "Sanrio
// characters" is left untouched (it has no iteration roman numeral).
const ITERATION_SUFFIX_RE = /\/[IVXLCDM]+$/i;

function buildSections(filtered: ICardData[], mode: SortMode, popularity: Map<string, ISkinPopularityInfo> | null, t: ViewerT): ISkinSection[] {
    const sortByDate = (a: ICardData, b: ICardData) => {
        const aTime = a.skin.displaySkin?.getTime ?? 0;
        const bTime = b.skin.displaySkin?.getTime ?? 0;
        if (aTime !== bTime) return bTime - aTime;
        const aSid = a.skin.displaySkin?.sortId ?? 0;
        const bSid = b.skin.displaySkin?.sortId ?? 0;
        if (aSid !== bSid) return bSid - aSid;
        return a.opName.localeCompare(b.opName);
    };

    if (mode === "date") {
        const all = [...filtered].sort(sortByDate);
        if (all.length === 0) return [];
        return [{ key: "all-by-date", title: t("profile.skins.section.all"), channel: "store", tag: null, sortIndex: 0, cards: all }];
    }

    if (mode === "popularity") {
        const ownerOf = (c: ICardData) => popularity?.get(c.skin.skinId)?.owners ?? 0;
        const all = [...filtered].sort((a, b) => {
            const diff = ownerOf(b) - ownerOf(a);
            if (diff !== 0) return diff;
            return sortByDate(a, b);
        });
        if (all.length === 0) return [];
        return [{ key: "all-by-popularity", title: t("profile.skins.section.all"), channel: "store", tag: null, sortIndex: 0, cards: all }];
    }

    // mode === "brand"
    const map = new Map<string, ISkinSection>();
    for (const c of filtered) {
        const ds = c.skin.displaySkin;
        const rawName = ds?.skinGroupName ?? t("profile.skins.section.other");
        const baseName = rawName.replace(ITERATION_SUFFIX_RE, "").trim() || rawName;
        const channel = classifyChannel(ds?.displayTagId);
        const key = `${channel}:${baseName}`;

        let section = map.get(key);
        if (!section) {
            section = {
                key,
                title: baseName,
                channel,
                tag: channelLabel(channel, t),
                sortIndex: ds?.skinGroupSortIndex ?? 0,
                cards: [],
            };
            map.set(key, section);
        } else if ((ds?.skinGroupSortIndex ?? 0) > section.sortIndex) {
            section.sortIndex = ds.skinGroupSortIndex ?? section.sortIndex;
        }
        section.cards.push(c);
    }

    for (const s of map.values()) s.cards.sort(sortByDate);

    return Array.from(map.values()).sort((a, b) => {
        const pa = CHANNEL_PRIORITY[a.channel];
        const pb = CHANNEL_PRIORITY[b.channel];
        if (pa !== pb) return pa - pb;
        if (a.sortIndex !== b.sortIndex) return b.sortIndex - a.sortIndex;
        return a.title.localeCompare(b.title);
    });
}

// ─── Pricing model ─────────────────────────────────────────────────────────

/** Per-skin Originite Prime cost overrides. Keyed by **skinId** (most specific -
 *  e.g. `char_002_amiya@witch#1`) or **skinGroupId** (e.g. `2024#witch` - applies
 *  to every skin in that iteration of a brand).
 *
 *  We need this because the AK client gamedata we ship (skin_table.json,
 *  shop_client_table.json) does NOT include per-skin OP prices - those are
 *  served by the live store API only. So there's no programmatic way to derive
 *  them from the data. Until we have a reliable source, populate this table
 *  with verified values per skin or per group.
 *
 *  Lookup order in `getSkinPrice`:
 *    1. exact `skinId` match
 *    2. `skinGroupId` match
 *    3. no entry → generic "Store" chip (no OP number) so we don't lie. */
const SKIN_PRICE_OVERRIDES: Record<string, number> = {
    // Examples (uncomment and fill in verified values):
    // "char_002_amiya@witch#1": 21,
    // "2024#witch": 21,
};

function getSkinPrice(skin: ISkinIndexEntry, t: ViewerT): ISkinPrice {
    const channel = classifyChannel(skin.displaySkin?.displayTagId);
    if (channel === "event" || channel === "is" || channel === "seasonal" || channel === "code-exchange") {
        const label = channel === "is" ? t("profile.skins.price.is") : t("profile.skins.price.free");
        return { kind: "free", label, tooltip: channelLabel(channel, t) };
    }
    if (channel === "special-pack") {
        return { kind: "bundle", label: t("profile.skins.price.bundle"), tooltip: t("profile.skins.price.bundle.tooltip") };
    }
    // Try per-skin then per-group override.
    const op = SKIN_PRICE_OVERRIDES[skin.skinId] ?? (skin.displaySkin?.skinGroupId ? SKIN_PRICE_OVERRIDES[skin.displaySkin.skinGroupId] : undefined);
    if (op != null) {
        return { kind: "paid", label: t("profile.skins.price.op", { op }), tooltip: t("profile.skins.price.store.tooltip") };
    }
    // Unknown store price - render no chip (label: null) rather than a noisy "Store" tag.
    return { kind: "store", label: null, tooltip: null };
}

interface ISectionHeaderProps {
    title: string;
    tag: string | null;
    count: number;
    color: string;
}

function SectionHeader({ title, tag, count, color }: ISectionHeaderProps) {
    const f = useFormatters();
    return (
        <div className="sticky top-0 z-10 -mx-3 flex items-center gap-2 border-border/40 border-b bg-background/95 px-3 py-1.5 backdrop-blur sm:-mx-4 sm:px-4">
            <h3 className="truncate font-heading font-semibold text-sm">{title}</h3>
            {tag && (
                <span className="rounded-full border px-1.5 py-px font-mono text-[9.5px] uppercase tracking-wider" style={{ borderColor: `color-mix(in oklch, ${color} 45%, transparent)`, color }}>
                    {tag}
                </span>
            )}
            <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">{f.number(count)}</span>
        </div>
    );
}
