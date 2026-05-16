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

export function SkinViewerDialog({ skins, ownedIds, profileOwnedCount, operatorsMap, color }: ISkinViewerDialogProps) {
    const [query, setQuery] = useState("");
    const deferredQuery = useDeferredValue(query);
    const [filter, setFilter] = useState<OwnershipFilter>("missing");

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
        const arr = allNonDefault.filter((s) => {
            const owned = ownedIds.has(s.skinId);
            if (filter === "owned" && !owned) return false;
            if (filter === "missing" && owned) return false;
            if (!q) return true;
            const opName = operatorsMap.get(s.charId)?.name?.toLowerCase() ?? "";
            const skinName = s.displaySkin?.skinName?.toLowerCase() ?? "";
            const groupName = s.displaySkin?.skinGroupName?.toLowerCase() ?? "";
            return opName.includes(q) || skinName.includes(q) || groupName.includes(q);
        });
        arr.sort((a, b) => {
            // Newest first by release date; fall back to sortId, then operator name
            // to keep groups of same-date skins ordered consistently.
            const aTime = a.displaySkin?.getTime ?? 0;
            const bTime = b.displaySkin?.getTime ?? 0;
            if (aTime !== bTime) return bTime - aTime;
            const sid = (b.displaySkin?.sortId ?? 0) - (a.displaySkin?.sortId ?? 0);
            if (sid !== 0) return sid;
            const aOp = operatorsMap.get(a.charId)?.name ?? "";
            const bOp = operatorsMap.get(b.charId)?.name ?? "";
            return aOp.localeCompare(bOp);
        });
        return arr;
    }, [allNonDefault, operatorsMap, deferredQuery, filter, ownedIds]);

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

                <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                    <FilterTabs color={color} counts={{ missing: missingCount, owned: ownedCount, all: totalCount }} onChange={setFilter} value={filter} />
                    <div className="relative flex items-center">
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

            {filtered.length === 0 ? (
                <div className="flex flex-1 flex-col items-center justify-center gap-2 p-8 text-center">
                    <p className="font-medium text-sm">No skins match your filters</p>
                    <p className="text-muted-foreground text-xs">{deferredQuery ? <>Try a different search or switch the ownership filter.</> : <>Try switching the ownership filter.</>}</p>
                </div>
            ) : (
                <ScrollArea className="min-h-0 flex-1">
                    <div className="grid grid-cols-3 gap-2 p-3 sm:grid-cols-4 sm:gap-2.5 sm:p-4 md:grid-cols-6 lg:grid-cols-8 xl:grid-cols-10 2xl:grid-cols-12">
                        {filtered.map((skin) => (
                            <SkinCard color={color} key={skin.skinId} op={operatorsMap.get(skin.charId)} owned={ownedIds.has(skin.skinId)} skin={skin} />
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

interface ISkinCardProps {
    skin: ISkin;
    op: IOperatorListItem | undefined;
    owned: boolean;
    color: string;
}

function SkinCard({ skin, op, owned, color }: ISkinCardProps) {
    const opName = op?.name ?? skin.charId;
    const skinName = skin.displaySkin?.skinName ?? skin.displaySkin?.skinGroupName ?? "Skin";

    return (
        <Dialog>
            <DialogTrigger aria-label={`View ${opName} · ${skinName}`} className={cn("group relative flex cursor-pointer flex-col overflow-hidden rounded-lg border bg-card text-left transition-all", "hover:-translate-y-0.5 hover:border-foreground/30 hover:shadow-md", owned ? "border-border/60" : "border-border")}>
                <div className="relative aspect-square w-full overflow-hidden bg-muted/30">
                    <img alt="" aria-hidden className={cn("h-full w-full object-cover object-center transition-transform duration-300 group-hover:scale-[1.04]", !owned && "opacity-60 saturate-50 group-hover:opacity-85 group-hover:saturate-75")} decoding="async" loading="lazy" src={getAvatarById(skin.skinId)} />
                    {!owned && <span aria-hidden className="pointer-events-none absolute inset-0 bg-gradient-to-t from-background/40 to-transparent" />}
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

    return (
        <DialogContent className="flex h-[92vh] max-h-[92vh] w-[min(960px,95vw)] max-w-[min(960px,95vw)] flex-col overflow-hidden p-0 sm:max-w-[min(960px,95vw)]" bottomStickOnMobile={false} showCloseButton>
            <DialogTitle className="sr-only">{`${opName} — ${skinName}`}</DialogTitle>
            <div className="grid min-h-0 flex-1 grid-cols-1 md:grid-cols-[5fr_4fr]">
                <div className="relative flex items-center justify-center overflow-hidden bg-gradient-to-b from-muted/20 to-muted/60 md:border-border/60 md:border-r">
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
                                <h3 className="break-words font-heading font-semibold text-lg leading-tight">{skinName}</h3>
                                {groupName && groupName !== skinName && <p className="truncate text-muted-foreground text-xs">{groupName}</p>}
                            </div>
                        </div>

                        <dl className="flex flex-col gap-3 text-sm">
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
