import { Check, Search, X } from "lucide-react";
import { useDeferredValue, useMemo, useState } from "react";
import { skinTexture } from "#/components/operators/detail/impl/assets";
import { Dialog, DialogClose, DialogContent, DialogTitle, DialogTrigger } from "#/components/ui/dialog";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { ScrollArea } from "#/components/ui/scroll-area";
import type { ISkin } from "#/lib/api/skins";
import { cn, getAvatarById } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";

type OwnershipFilter = "all" | "missing" | "owned";
type SortMode = "brand" | "date";

interface ISkinViewerDialogProps {
    skins: ISkin[];
    ownedIds: Set<string>;
    /** Authoritative owned count from the user profile. Used for the header display
     *  so it's correct even before the per-skin ownership list finishes loading. */
    profileOwnedCount: number;
    operatorsMap: Map<string, IOperatorListItem>;
    color: string;
}

const FILTER_TABS: { id: OwnershipFilter; label: string }[] = [
    { id: "all", label: "All" },
    { id: "owned", label: "Owned" },
    { id: "missing", label: "Missing" },
];

const SORT_TABS: { id: SortMode; label: string }[] = [
    { id: "brand", label: "Brand" },
    { id: "date", label: "Date" },
];

export function SkinViewerDialog({ skins, ownedIds, profileOwnedCount, operatorsMap, color }: ISkinViewerDialogProps) {
    const [query, setQuery] = useState("");
    const deferredQuery = useDeferredValue(query);
    const [filter, setFilter] = useState<OwnershipFilter>("missing");
    const [sort, setSort] = useState<SortMode>("brand");

    const allNonDefault = useMemo(() => skins.filter((s) => s.skinId?.includes("@")), [skins]);

    // Per-skin enumeration may still be loading; trust the profile count for the
    // header total. Fall back to the intersection only if the profile count is
    // missing (legacy data).
    const enumeratedOwnedCount = useMemo(() => {
        let n = 0;
        for (const s of allNonDefault) if (ownedIds.has(s.skinId)) n++;
        return n;
    }, [allNonDefault, ownedIds]);
    const ownedCount = profileOwnedCount > 0 ? profileOwnedCount : enumeratedOwnedCount;

    const totalCount = allNonDefault.length;
    const missingCount = Math.max(0, totalCount - ownedCount);

    const filtered = useMemo(() => {
        const q = deferredQuery.trim().toLowerCase();
        return allNonDefault.filter((s) => {
            const owned = ownedIds.has(s.skinId);
            if (filter === "owned" && !owned) return false;
            if (filter === "missing" && owned) return false;
            if (!q) return true;
            const opName = operatorsMap.get(s.charId)?.name?.toLowerCase() ?? "";
            const skinName = s.displaySkin?.skinName?.toLowerCase() ?? "";
            const groupName = s.displaySkin?.skinGroupName?.toLowerCase() ?? "";
            return opName.includes(q) || skinName.includes(q) || groupName.includes(q);
        });
    }, [allNonDefault, operatorsMap, deferredQuery, filter, ownedIds]);

    const sections = useMemo(() => buildSections(filtered, operatorsMap, sort), [filtered, operatorsMap, sort]);
    const totalFiltered = filtered.length;

    return (
        <DialogContent className="flex h-[95vh] max-h-[95vh] w-[95vw] max-w-[95vw] flex-col overflow-hidden p-0 sm:max-w-[95vw]" bottomStickOnMobile={false} showCloseButton>
            <DialogTitle className="sr-only">Skin Collection</DialogTitle>

            <header className="flex shrink-0 flex-col gap-3 border-border/60 border-b bg-card/60 p-4 backdrop-blur sm:p-5">
                <div className="flex flex-wrap items-baseline gap-x-4 gap-y-1">
                    <h2 className="font-heading font-semibold text-lg leading-none sm:text-xl">Skin Collection</h2>
                    <p className="font-mono text-[11px] text-muted-foreground tabular-nums">
                        <span className="font-semibold" style={{ color }}>
                            {missingCount.toLocaleString()}
                        </span>{" "}
                        missing · <span className="font-semibold text-foreground">{ownedCount.toLocaleString()}</span> owned · {totalCount.toLocaleString()} total
                    </p>
                </div>

                <div className="flex flex-col gap-2 sm:flex-row sm:flex-wrap sm:items-center sm:gap-3">
                    <FilterTabs color={color} counts={{ missing: missingCount, owned: ownedCount, all: totalCount }} onChange={setFilter} value={filter} />
                    <SortTabs onChange={setSort} value={sort} />
                    <div className="relative flex items-center sm:ml-auto">
                        <Search aria-hidden className="absolute left-2.5 h-3.5 w-3.5 text-muted-foreground/60" />
                        <input
                            aria-label="Search skins"
                            className="w-full rounded-md border border-border bg-background py-1.5 pr-8 pl-8 text-sm outline-none transition-colors placeholder:text-muted-foreground/50 focus:border-foreground/30 sm:w-64"
                            onChange={(e) => setQuery(e.target.value)}
                            placeholder="Search by operator or skin…"
                            type="search"
                            value={query}
                        />
                        {query && (
                            <button aria-label="Clear search" className="absolute right-1.5 flex h-6 w-6 cursor-pointer items-center justify-center rounded-full text-muted-foreground hover:bg-muted hover:text-foreground" onClick={() => setQuery("")} type="button">
                                <X className="h-3 w-3" />
                            </button>
                        )}
                    </div>
                </div>
            </header>

            {totalFiltered === 0 ? (
                <div className="flex flex-1 flex-col items-center justify-center gap-2 p-8 text-center">
                    <p className="font-medium text-sm">No skins match your filters</p>
                    <p className="text-muted-foreground text-xs">{deferredQuery ? <>Try a different search or switch the ownership filter.</> : <>Try switching the ownership filter.</>}</p>
                </div>
            ) : (
                <ScrollArea className="min-h-0 flex-1">
                    <div className="flex flex-col gap-5 px-3 pt-3 pb-4 sm:gap-6 sm:px-4 sm:pb-5">
                        {sections.map((section) => (
                            <section className="flex flex-col gap-2" key={section.key}>
                                {sort !== "date" && <SectionHeader color={color} count={section.skins.length} tag={section.tag} title={section.title} />}
                                <div className="grid grid-cols-3 gap-2 sm:grid-cols-4 sm:gap-2.5 md:grid-cols-6 lg:grid-cols-8 xl:grid-cols-10 2xl:grid-cols-12">
                                    {section.skins.map((skin) => (
                                        <SkinCard color={color} key={skin.skinId} op={operatorsMap.get(skin.charId)} owned={ownedIds.has(skin.skinId)} skin={skin} />
                                    ))}
                                </div>
                            </section>
                        ))}
                    </div>
                </ScrollArea>
            )}
        </DialogContent>
    );
}

interface IFilterTabsProps {
    value: OwnershipFilter;
    onChange: (v: OwnershipFilter) => void;
    counts: Record<OwnershipFilter, number>;
    color: string;
}

function FilterTabs({ value, onChange, counts, color }: IFilterTabsProps) {
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
                        {tab.label}
                        <span className="font-semibold tabular-nums">{counts[tab.id].toLocaleString()}</span>
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
    return (
        <div className="inline-flex shrink-0 items-center gap-0.5 rounded-md border border-border bg-muted/40 p-0.5">
            <span className="px-1.5 font-mono text-[10px] text-muted-foreground uppercase tracking-wider">Sort</span>
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
                        {tab.label}
                    </button>
                );
            })}
        </div>
    );
}

interface ISkinCardProps {
    skin: ISkin;
    op: IOperatorListItem | undefined;
    owned: boolean;
    color: string;
}

function SkinCard({ skin, op, owned, color }: ISkinCardProps) {
    const opName = op?.name ?? skin.charId;
    const skinName = skin.displaySkin?.skinName ?? skin.displaySkin?.skinGroupName ?? "Skin";
    const price = getSkinPrice(skin);

    return (
        <Dialog>
            <DialogTrigger aria-label={`View ${opName} · ${skinName}`} className={cn("group relative flex cursor-pointer flex-col overflow-hidden rounded-lg border bg-card text-left transition-all", "hover:-translate-y-0.5 hover:border-foreground/30 hover:shadow-md", owned ? "border-border/60" : "border-border")}>
                <div className="relative aspect-square w-full overflow-hidden bg-muted/30">
                    <img alt="" aria-hidden className={cn("h-full w-full object-cover object-center transition-transform duration-300 group-hover:scale-[1.04]", !owned && "opacity-60 saturate-50 group-hover:opacity-85 group-hover:saturate-75")} decoding="async" loading="lazy" src={getAvatarById(skin.skinId)} />
                    {!owned && <span aria-hidden className="pointer-events-none absolute inset-0 bg-linear-to-t from-background/40 to-transparent" />}
                    {price.label && (
                        <span className="absolute top-1 left-1">
                            <PriceChip price={price} />
                        </span>
                    )}
                    <span className="absolute top-1 right-1">
                        <OwnershipBadge color={color} owned={owned} />
                    </span>
                </div>
                <div className="flex min-w-0 flex-col gap-0.5 px-1.5 py-1.5">
                    <span className="truncate font-medium text-[11px] leading-tight">{skinName}</span>
                    <span className="truncate text-[10px] text-muted-foreground leading-tight">{opName}</span>
                </div>
            </DialogTrigger>
            <SkinDetailDialog color={color} op={op} owned={owned} skin={skin} />
        </Dialog>
    );
}

function PriceChip({ price }: { price: ISkinPrice }) {
    const isFree = price.kind === "free";
    return (
        <span className={cn("flex items-center gap-0.5 rounded-full px-1.5 py-px font-mono font-semibold text-[9px] uppercase tracking-wider shadow-sm backdrop-blur-sm", isFree ? "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400" : "bg-background/85 text-foreground")} title={price.tooltip ?? undefined}>
            {price.label}
        </span>
    );
}

function OwnershipBadge({ owned, color }: { owned: boolean; color: string }) {
    if (owned) {
        return (
            <span aria-label="Owned" className="flex h-5 w-5 items-center justify-center rounded-full bg-emerald-500/95 text-white shadow-sm" role="img">
                <Check aria-hidden className="h-3 w-3" strokeWidth={3} />
            </span>
        );
    }
    return <span aria-label="Missing" className="block h-5 w-5 rounded-full border bg-background/85 backdrop-blur-sm" role="img" style={{ borderColor: `color-mix(in oklch, ${color} 50%, transparent)` }} />;
}

interface ISkinDetailDialogProps {
    skin: ISkin;
    op: IOperatorListItem | undefined;
    owned: boolean;
    color: string;
}

function SkinDetailDialog({ skin, op, owned, color }: ISkinDetailDialogProps) {
    const opName = op?.name ?? skin.charId;
    const ds = skin.displaySkin;
    const skinName = ds?.skinName ?? ds?.skinGroupName ?? "Skin";
    const groupName = ds?.skinGroupName;
    const description = ds?.description ?? ds?.content ?? null;
    const dialog = ds?.dialog ?? null;
    const usage = ds?.usage ?? null;
    const obtain = ds?.obtainApproach ?? null;
    const designers = ds?.designerList ?? null;
    const drawers = ds?.drawerList ?? null;
    const releaseTs = ds?.getTime ? ds.getTime * 1000 : null;
    const heroUrl = skinTexture(skin.charId, skin.skinId);
    const fallbackUrl = getAvatarById(skin.skinId);
    const price = getSkinPrice(skin);

    return (
        <DialogContent className="flex h-[92vh] max-h-[92vh] w-[min(960px,95vw)] max-w-[min(960px,95vw)] flex-col overflow-hidden p-0 sm:max-w-[min(960px,95vw)]" bottomStickOnMobile={false} showCloseButton>
            <DialogTitle className="sr-only">{`${opName} — ${skinName}`}</DialogTitle>
            <div className="grid min-h-0 flex-1 grid-cols-1 md:grid-cols-[5fr_4fr]">
                <div className="relative flex items-center justify-center overflow-hidden bg-linear-to-b from-muted/20 to-muted/60 md:border-border/60 md:border-r">
                    <img alt={`${opName} ${skinName}`} className="h-full w-full object-contain object-bottom" decoding="async" loading="lazy" onError={(e) => ((e.target as HTMLImageElement).src = fallbackUrl)} src={heroUrl} />
                    <span className="absolute top-3 left-3">
                        <OwnershipBadge color={color} owned={owned} />
                    </span>
                </div>
                <ScrollArea className="min-h-0">
                    <div className="flex flex-col gap-4 p-5 sm:p-6">
                        <div className="flex items-start gap-3">
                            <span className="flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-muted/40 font-semibold">
                                <OperatorAvatar charId={op?.id ?? skin.charId} name={opName} />
                            </span>
                            <div className="min-w-0 flex-1">
                                <p className="truncate font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{opName}</p>
                                <h3 className="wrap-break-word font-heading font-semibold text-lg leading-tight">{skinName}</h3>
                                {groupName && groupName !== skinName && <p className="truncate text-muted-foreground text-xs">{groupName}</p>}
                            </div>
                        </div>

                        <dl className="flex flex-col gap-3 text-sm">
                            {price.label && (
                                <DetailRow label="Price">
                                    <span className={cn("font-semibold", price.kind === "free" && "text-emerald-600 dark:text-emerald-400")}>{price.label}</span>
                                    {price.tooltip && <span className="ml-2 text-muted-foreground text-xs">— {price.tooltip}</span>}
                                </DetailRow>
                            )}
                            {obtain && <DetailRow label="Obtain">{obtain}</DetailRow>}
                            {usage && <DetailRow label="Usage">{usage}</DetailRow>}
                            {description && <DetailRow label="Description">{description}</DetailRow>}
                            {dialog && (
                                <DetailRow label="Dialog">
                                    <q className="italic">{dialog}</q>
                                </DetailRow>
                            )}
                            {(drawers?.length || designers?.length) && (
                                <DetailRow label="Credits">
                                    {drawers?.length ? <span>Art: {drawers.join(", ")}</span> : null}
                                    {drawers?.length && designers?.length ? " · " : null}
                                    {designers?.length ? <span>Design: {designers.join(", ")}</span> : null}
                                </DetailRow>
                            )}
                            {releaseTs && <DetailRow label="Released">{new Date(releaseTs).toLocaleDateString(undefined, { year: "numeric", month: "long", day: "numeric" })}</DetailRow>}
                        </dl>

                        <DialogClose className="mt-auto cursor-pointer rounded-md border border-border bg-muted/40 px-3 py-2 font-medium text-xs transition-colors hover:bg-muted md:hidden">Back to collection</DialogClose>
                    </div>
                </ScrollArea>
            </div>
        </DialogContent>
    );
}

function DetailRow({ label, children }: { label: string; children: React.ReactNode }) {
    return (
        <div className="space-y-1">
            <dt className="font-mono font-semibold text-[10px] text-muted-foreground uppercase tracking-[0.12em]">{label}</dt>
            <dd className="text-foreground/85 text-sm leading-relaxed">{children}</dd>
        </div>
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
    skins: ISkin[];
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

const CHANNEL_LABEL: Record<SectionChannel, string | null> = {
    collab: "Collab",
    store: null, // implicit — main paid brands are the baseline, no chip needed
    seasonal: "Seasonal",
    "special-pack": "Special Pack",
    is: "IS Reward",
    event: "Event Reward",
    "code-exchange": "Code Exchange",
};

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

function buildSections(filtered: ISkin[], operatorsMap: Map<string, IOperatorListItem>, mode: SortMode): ISkinSection[] {
    const sortByDate = (a: ISkin, b: ISkin) => {
        const aTime = a.displaySkin?.getTime ?? 0;
        const bTime = b.displaySkin?.getTime ?? 0;
        if (aTime !== bTime) return bTime - aTime;
        const aSid = a.displaySkin?.sortId ?? 0;
        const bSid = b.displaySkin?.sortId ?? 0;
        if (aSid !== bSid) return bSid - aSid;
        const aOp = operatorsMap.get(a.charId)?.name ?? "";
        const bOp = operatorsMap.get(b.charId)?.name ?? "";
        return aOp.localeCompare(bOp);
    };

    if (mode === "date") {
        const all = [...filtered].sort(sortByDate);
        if (all.length === 0) return [];
        return [{ key: "all-by-date", title: "All Skins", channel: "store", tag: null, sortIndex: 0, skins: all }];
    }

    // mode === "brand"
    const map = new Map<string, ISkinSection>();
    for (const s of filtered) {
        const ds = s.displaySkin;
        const rawName = ds?.skinGroupName ?? "Other";
        const baseName = rawName.replace(ITERATION_SUFFIX_RE, "").trim() || rawName;
        const channel = classifyChannel(ds?.displayTagId);
        const key = `${channel}:${baseName}`;

        let section = map.get(key);
        if (!section) {
            section = {
                key,
                title: baseName,
                channel,
                tag: CHANNEL_LABEL[channel],
                sortIndex: ds?.skinGroupSortIndex ?? 0,
                skins: [],
            };
            map.set(key, section);
        } else if ((ds?.skinGroupSortIndex ?? 0) > section.sortIndex) {
            section.sortIndex = ds.skinGroupSortIndex ?? section.sortIndex;
        }
        section.skins.push(s);
    }

    for (const s of map.values()) s.skins.sort(sortByDate);

    return Array.from(map.values()).sort((a, b) => {
        const pa = CHANNEL_PRIORITY[a.channel];
        const pb = CHANNEL_PRIORITY[b.channel];
        if (pa !== pb) return pa - pb;
        if (a.sortIndex !== b.sortIndex) return b.sortIndex - a.sortIndex;
        return a.title.localeCompare(b.title);
    });
}

// ─── Pricing model ─────────────────────────────────────────────────────────

interface ISkinPrice {
    kind: "paid" | "free" | "bundle" | "store";
    /** Short display label for chips, e.g. "18 OP" or "Free". `null` = don't render a chip. */
    label: string | null;
    /** Longer hover / secondary text — e.g. acquisition method. */
    tooltip: string | null;
}

/** Per-skin Originite Prime cost overrides. Keyed by **skinId** (most specific —
 *  e.g. `char_002_amiya@witch#1`) or **skinGroupId** (e.g. `2024#witch` — applies
 *  to every skin in that iteration of a brand).
 *
 *  We need this because the AK client gamedata we ship (skin_table.json,
 *  shop_client_table.json) does NOT include per-skin OP prices — those are
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

function getSkinPrice(skin: ISkin): ISkinPrice {
    const channel = classifyChannel(skin.displaySkin?.displayTagId);
    if (channel === "event" || channel === "is" || channel === "seasonal" || channel === "code-exchange") {
        const label = channel === "is" ? "IS" : "Free";
        return { kind: "free", label, tooltip: CHANNEL_LABEL[channel] ?? null };
    }
    if (channel === "special-pack") {
        return { kind: "bundle", label: "Bundle", tooltip: "Obtain from Special Pack" };
    }
    // Try per-skin then per-group override.
    const op = SKIN_PRICE_OVERRIDES[skin.skinId] ?? (skin.displaySkin?.skinGroupId ? SKIN_PRICE_OVERRIDES[skin.displaySkin.skinGroupId] : undefined);
    if (op != null) {
        return { kind: "paid", label: `${op} OP`, tooltip: "Outfit Store" };
    }
    // Unknown store price — render no chip (label: null) rather than a noisy "Store" tag.
    return { kind: "store", label: null, tooltip: null };
}

interface ISectionHeaderProps {
    title: string;
    tag: string | null;
    count: number;
    color: string;
}

function SectionHeader({ title, tag, count, color }: ISectionHeaderProps) {
    return (
        <div className="sticky top-0 z-10 -mx-3 flex items-center gap-2 border-border/40 border-b bg-background/95 px-3 py-1.5 backdrop-blur sm:-mx-4 sm:px-4">
            <h3 className="truncate font-heading font-semibold text-sm">{title}</h3>
            {tag && (
                <span className="rounded-full border px-1.5 py-px font-mono text-[9.5px] uppercase tracking-wider" style={{ borderColor: `color-mix(in oklch, ${color} 45%, transparent)`, color }}>
                    {tag}
                </span>
            )}
            <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">{count.toLocaleString()}</span>
        </div>
    );
}
