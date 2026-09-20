import { Check, ChevronDown, Search, X } from "lucide-react";
import { useDeferredValue, useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { ItemIcon } from "#/components/user/profile/impl/components/tabs/Items/ItemIcon";
import type { IMaterials } from "#/lib/api/materials";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { LEADERBOARD_SORTS, type LeaderboardSort, type Ranking } from "../constants";
import type { messages as constantsMessages } from "../constants.messages";
import { FEATURED_ITEMS } from "../inventory.constants";
import { resolveCatalogItem, toIconEntry } from "../inventory.helpers";
import type { ICatalogItem } from "../inventory.types";
import type { messages } from "./RankByPicker.messages";

/** The sort labels in `constants.messages.ts` are rendered here too. */
type PickerT = TypedT<typeof messages & typeof constantsMessages>;

interface IRankByPickerProps {
    ranking: Ranking;
    onRanking: (next: Ranking) => void;
    catalog: ICatalogItem[];
    materials: IMaterials | undefined;
    /** `toolbar` is the labelled control; `header` is the compact trigger in the table's value column. */
    variant: "toolbar" | "header";
    className?: string;
}

const SEARCH_LIMIT = 60;

/** Name of the active ranking for the trigger, and the item behind it. */
function useRankingLabel(ranking: Ranking, catalog: ICatalogItem[], materials: IMaterials | undefined): { label: string; item: ICatalogItem | null } {
    const t: PickerT = useT("user");
    return useMemo(() => {
        if (ranking.kind === "score") {
            const key = LEADERBOARD_SORTS.find((s) => s.value === ranking.sort)?.labelKey ?? "leaderboard.sort.total";
            return { label: t(key), item: null };
        }
        const item = resolveCatalogItem(catalog, ranking.item, materials);
        return { label: item.name, item };
    }, [ranking, catalog, materials, t]);
}

export function RankByPicker({ ranking, onRanking, catalog, materials, variant, className }: IRankByPickerProps) {
    const t: PickerT = useT("user");
    const [open, setOpen] = useState(false);
    const [query, setQuery] = useState("");
    const deferred = useDeferredValue(query.trim().toLowerCase());
    const { label, item: activeItem } = useRankingLabel(ranking, catalog, materials);

    const scoreOptions = useMemo(() => LEADERBOARD_SORTS.map((s) => ({ value: s.value, label: t(s.labelKey) })).filter((o) => !deferred || o.label.toLowerCase().includes(deferred)), [deferred, t]);
    const itemOptions = useMemo<ICatalogItem[]>(() => {
        if (!deferred) {
            // Before a search the list is the featured set, so the first
            // things a visitor sees are the currencies they came to compare. A
            // featured item nobody holds yet (Originite Prime is only stored
            // for players who synced after 2026-09-15) still appears, at 0.
            return FEATURED_ITEMS.map((id) => resolveCatalogItem(catalog, id, materials));
        }
        return catalog.filter((c) => c.name.toLowerCase().includes(deferred) || c.item_id.toLowerCase().includes(deferred)).slice(0, SEARCH_LIMIT);
    }, [deferred, catalog, materials]);

    const choose = (next: Ranking) => {
        onRanking(next);
        setOpen(false);
        setQuery("");
    };

    const trigger =
        variant === "toolbar" ? (
            <PopoverTrigger aria-label={t("leaderboard.rankBy.aria")} className={cn("inline-flex h-9 min-w-0 cursor-pointer items-center gap-2 rounded-lg border border-input bg-card pr-2.5 pl-2 font-medium font-sans text-foreground text-xs leading-none transition-colors hover:border-foreground/20 sm:h-8", className)}>
                {activeItem ? <ItemIcon item={toIconEntry(activeItem)} size={20} className="rounded" /> : null}
                <span className="shrink-0 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">{t("leaderboard.rankBy.label")}</span>
                <span className="min-w-0 truncate font-semibold text-foreground">{label}</span>
                <ChevronDown className="size-3 shrink-0 opacity-70" aria-hidden />
            </PopoverTrigger>
        ) : (
            <PopoverTrigger aria-label={t("leaderboard.rankBy.aria")} className={cn("inline-flex min-w-0 max-w-full shrink-0 cursor-pointer items-center gap-1 whitespace-nowrap font-medium font-mono text-[11px] text-foreground uppercase leading-none tracking-[0.16em] transition-colors hover:text-primary", className)}>
                <span className="truncate">{label}</span>
                <ChevronDown className="size-3 shrink-0 opacity-70" aria-hidden />
            </PopoverTrigger>
        );

    return (
        <Popover open={open} onOpenChange={setOpen}>
            {trigger}
            <PopoverPopup align={variant === "toolbar" ? "start" : "end"} className="w-[min(400px,calc(100vw-2rem))] p-0">
                <div className="border-border border-b p-2">
                    <InputGroup>
                        <InputGroupAddon>
                            <Search aria-hidden="true" />
                        </InputGroupAddon>
                        <InputGroupInput autoFocus value={query} onChange={(e) => setQuery(e.target.value)} placeholder={t("leaderboard.rankBy.search.placeholder")} aria-label={t("leaderboard.rankBy.search.label")} />
                        <InputGroupAddon align="inline-end">
                            {query ? (
                                <Button variant="ghost" size="icon-xs" onClick={() => setQuery("")} aria-label={t("leaderboard.rankBy.search.clear")}>
                                    <X aria-hidden="true" />
                                </Button>
                            ) : null}
                        </InputGroupAddon>
                    </InputGroup>
                </div>
                <div className="max-h-[min(420px,60dvh)] overflow-y-auto p-1">
                    {scoreOptions.length === 0 && itemOptions.length === 0 ? <p className="m-0 px-3 py-6 text-center font-sans text-muted-foreground text-sm">{t("leaderboard.rankBy.empty")}</p> : null}

                    {scoreOptions.length > 0 ? (
                        <section aria-label={t("leaderboard.rankBy.group.score")}>
                            <GroupHeading>{t("leaderboard.rankBy.group.score")}</GroupHeading>
                            {/* Seven metrics as chips, not rows: the Inventory group
                                below is what most visitors open this for, and it
                                must be visible without scrolling the popup. */}
                            <ul className="m-0 flex list-none flex-wrap gap-1.5 px-1.5 pb-1.5">
                                {scoreOptions.map((opt) => {
                                    const active = ranking.kind === "score" && ranking.sort === opt.value;
                                    return (
                                        <li key={opt.value}>
                                            <button
                                                type="button"
                                                aria-pressed={active}
                                                onClick={() => choose({ kind: "score", sort: opt.value as LeaderboardSort })}
                                                className={cn(
                                                    "inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 font-medium font-sans text-xs leading-none transition-colors",
                                                    active ? "border-primary/40 bg-[color-mix(in_srgb,var(--primary)_10%,transparent)] text-primary" : "border-input bg-card text-foreground hover:border-foreground/20",
                                                )}
                                            >
                                                {active ? <Check className="size-3" aria-hidden /> : null}
                                                {opt.label}
                                            </button>
                                        </li>
                                    );
                                })}
                            </ul>
                        </section>
                    ) : null}

                    {itemOptions.length > 0 ? (
                        <section aria-label={t("leaderboard.rankBy.group.items")} className={scoreOptions.length > 0 ? "mt-1 border-border/70 border-t pt-1" : undefined}>
                            <GroupHeading hint={!deferred && catalog.length > 0 ? t("leaderboard.rankBy.group.items.hint", { count: catalog.length }) : undefined}>{t("leaderboard.rankBy.group.items")}</GroupHeading>
                            <ul className="m-0 list-none p-0">
                                {itemOptions.map((c) => {
                                    const active = ranking.kind === "item" && ranking.item === c.item_id;
                                    return (
                                        <li key={c.item_id}>
                                            <Option active={active} onClick={() => choose({ kind: "item", item: c.item_id })}>
                                                <ItemIcon item={toIconEntry(c)} size={28} className="rounded-md" />
                                                <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                                                    <span className="truncate">{c.name}</span>
                                                    <span className="truncate font-mono text-[10.5px] text-muted-foreground leading-none">{t("leaderboard.rankBy.holders", { count: c.holders })}</span>
                                                </span>
                                            </Option>
                                        </li>
                                    );
                                })}
                            </ul>
                        </section>
                    ) : null}
                </div>
            </PopoverPopup>
        </Popover>
    );
}

function GroupHeading({ children, hint }: { children: React.ReactNode; hint?: string }) {
    return (
        <div className="flex items-baseline justify-between gap-2 px-2 pt-1.5 pb-1">
            <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.16em]">{children}</span>
            {hint ? <span className="min-w-0 truncate font-sans text-[11px] text-muted-foreground leading-none">{hint}</span> : null}
        </div>
    );
}

function Option({ active, onClick, children }: { active: boolean; onClick: () => void; children: React.ReactNode }) {
    return (
        <button
            type="button"
            onClick={onClick}
            aria-pressed={active}
            className={cn("flex min-h-10 w-full cursor-pointer items-center gap-2.5 rounded-md px-2 py-1.5 text-left font-sans text-[13px] text-foreground leading-tight transition-colors hover:bg-accent", active && "bg-[color-mix(in_srgb,var(--primary)_8%,transparent)] font-semibold text-primary")}
        >
            {children}
            {active ? <Check className="size-3.5 shrink-0" aria-hidden /> : <span className="size-3.5 shrink-0" aria-hidden />}
        </button>
    );
}
