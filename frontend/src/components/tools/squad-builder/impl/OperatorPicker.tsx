import { X } from "lucide-react";
import * as React from "react";
import { ClassIcon, SubProfessionIcon } from "#/components/operators/list/impl/components/Icons";
import { PROFESSION_ORDER } from "#/components/operators/list/impl/constants";
import { eliteIcon, potentialIcon, specializedIcon } from "#/components/tools/shared/icons";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { Kbd } from "#/components/ui/kbd";
import { Label } from "#/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Switch } from "#/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { searchAndRank } from "#/lib/search/fuzzy";
import { archetypeLabel, cn, getAvatarById, professionLabel, rarityStarColor } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import { advancedModules, type IRosterBuild, isCurrentForm, moduleTypeLabel, rosterModuleRanks, SUPPORT_SEAT } from "../model";
import type { messages } from "./OperatorPicker.messages";

/** A filter control is 32-36 px tall with a fine pointer; on a touch screen it takes the full 44 px. */
const TOUCH_ITEM = "pointer-coarse:h-11 pointer-coarse:min-w-11";

const SORTS = ["rarity", "name", "cost", "level"] as const;
type SortKey = (typeof SORTS)[number];

type PickerT = TypedT<typeof messages>;

export interface IOperatorPickerProps {
    /** 0-11, or `SUPPORT_SEAT` for the support picker. */
    seat: number;
    operators: readonly IOperatorIndexEntry[];
    /** By operator id, every form of a multi-form operator included (`rosterBuildsOf`). */
    roster: ReadonlyMap<string, IRosterBuild>;
    /** `char_id` -> seat it already sits in (`SUPPORT_SEAT` for the support). */
    seated: ReadonlyMap<string, number>;
    onPick: (charId: string) => void;
    onClose: () => void;
}

/**
 * Same-named operators kept together in the order they came, the form the player has in use
 * first among them. Amiya's three forms tie on rarity and name, so without this the Caster
 * id wins every tie and is the one a player picks by default.
 */
function currentFormsFirst(list: readonly IOperatorIndexEntry[], roster: ReadonlyMap<string, IRosterBuild>): IOperatorIndexEntry[] {
    const firstAt = new Map<string, number>();
    list.forEach((op, i) => {
        if (!firstAt.has(op.name)) firstAt.set(op.name, i);
    });
    const rank = (op: IOperatorIndexEntry) => (isCurrentForm(op.id, roster.get(op.id)) ? 0 : 1);
    return list
        .map((op, i) => ({ op, i }))
        .sort((a, b) => (firstAt.get(a.op.name) ?? a.i) - (firstAt.get(b.op.name) ?? b.i) || rank(a.op) - rank(b.op) || a.i - b.i)
        .map((x) => x.op);
}

export function OperatorPicker({ seat, operators, roster, seated, onPick, onClose }: IOperatorPickerProps): React.ReactElement {
    const t: PickerT = useT("tools");
    const { collator } = useFormatters();
    const isSupport = seat === SUPPORT_SEAT;
    const hasRoster = roster.size > 0;
    const [query, setQuery] = React.useState("");
    const [ownedOnly, setOwnedOnly] = React.useState(hasRoster);
    const [cls, setCls] = React.useState<string>("all");
    const [branch, setBranch] = React.useState<string>("all");
    const [rarity, setRarity] = React.useState<string>("all");
    const [sort, setSort] = React.useState<SortKey>("rarity");
    const searchRef = React.useRef<HTMLInputElement>(null);
    const gridRef = React.useRef<HTMLUListElement>(null);
    const [active, setActive] = React.useState(0);

    // The roster can arrive after the picker opens; follow it until the player touches the switch.
    const touchedOwned = React.useRef(false);
    React.useEffect(() => {
        if (!touchedOwned.current) setOwnedOnly(hasRoster);
    }, [hasRoster]);

    React.useEffect(() => {
        searchRef.current?.focus({ preventScroll: true });
    }, []);

    const showOwnedFilter = !isSupport && hasRoster;
    const ownedFilter = showOwnedFilter && ownedOnly;
    const sortable = SORTS.filter((s) => s !== "level" || (hasRoster && !isSupport));

    // The star filter offers the rarities the index has, highest first.
    const rarities = React.useMemo(() => [...new Set(operators.map((op) => op.rarity))].sort((a, b) => b - a), [operators]);

    // The branches of the chosen class, by name, with the icon id each one draws.
    const branches = React.useMemo(() => {
        if (cls === "all") return [];
        const seen = new Map<string, string>();
        for (const op of operators) if (op.profession === cls && !seen.has(op.subProfessionId)) seen.set(op.subProfessionId, archetypeLabel(op));
        return [...seen].map(([id, label]) => ({ id, label })).sort((a, b) => collator.compare(a.label, b.label));
    }, [operators, cls, collator]);

    const pool = React.useMemo(() => {
        const professions = new Set<string>(PROFESSION_ORDER);
        return operators.filter((op) => {
            if (!professions.has(op.profession)) return false;
            const owned = roster.has(op.id);
            if (op.isNotObtainable && !owned) return false;
            if (ownedFilter && !owned) return false;
            if (cls !== "all" && op.profession !== cls) return false;
            if (branch !== "all" && op.subProfessionId !== branch) return false;
            if (rarity !== "all" && op.rarity !== Number(rarity)) return false;
            return true;
        });
    }, [operators, roster, ownedFilter, cls, branch, rarity]);

    const shown = React.useMemo(() => {
        if (query.trim()) {
            const ranked = searchAndRank(query, pool, (op) => {
                const cl = professionLabel(op);
                const br = archetypeLabel(op);
                // "amiya medic" finds the Medic form by name; "incantation" finds it as a weaker match.
                return { name: op.name, aliases: [op.appellation, `${op.name} ${cl}`, `${op.name} ${br}`], extra: `${cl} ${br}` };
            });
            return currentFormsFirst(
                ranked.map((r) => r.item),
                roster,
            );
        }
        const byName = (a: IOperatorIndexEntry, b: IOperatorIndexEntry) => collator.compare(a.name, b.name);
        const byRarity = (a: IOperatorIndexEntry, b: IOperatorIndexEntry) => b.rarity - a.rarity;
        const list = [...pool];
        if (sort === "name") list.sort((a, b) => byName(a, b) || byRarity(a, b));
        else if (sort === "cost") list.sort((a, b) => a.stats.cost - b.stats.cost || byRarity(a, b) || byName(a, b));
        else if (sort === "level") {
            // Owned first, by promotion then level; the rest keep the rarity order.
            const key = (op: IOperatorIndexEntry) => {
                const b = roster.get(op.id);
                return b ? b.elite * 1000 + b.level : -1;
            };
            list.sort((a, b) => key(b) - key(a) || byRarity(a, b) || byName(a, b));
        } else list.sort((a, b) => byRarity(a, b) || byName(a, b));
        return currentFormsFirst(list, roster);
    }, [pool, query, sort, roster, collator]);

    // A new result list starts the keyboard cursor on its first tile.
    const [cursorList, setCursorList] = React.useState(shown);
    if (cursorList !== shown) {
        setCursorList(shown);
        setActive(0);
    }

    const items = () => Array.from(gridRef.current?.querySelectorAll<HTMLButtonElement>("[data-picker-item]") ?? []);
    const focusItem = (i: number) => {
        const list = items();
        if (list.length === 0) return;
        const next = Math.max(0, Math.min(list.length - 1, i));
        setActive(next);
        list[next]?.focus();
    };
    /** Tiles per row, read from the layout: the grid's column count changes with its width. */
    const columns = () => {
        const list = items();
        const top = list[0]?.getBoundingClientRect().top;
        if (top === undefined) return 1;
        let n = 0;
        for (const el of list) {
            if (Math.abs(el.getBoundingClientRect().top - top) > 2) break;
            n++;
        }
        return Math.max(1, n);
    };

    const pickable = (op: IOperatorIndexEntry) => !seated.has(op.id);

    const onGridKeyDown = (e: React.KeyboardEvent<HTMLUListElement>) => {
        const cols = columns();
        const last = shown.length - 1;
        let next: number | null = null;
        if (e.key === "ArrowRight") next = active + 1;
        else if (e.key === "ArrowLeft") next = active - 1;
        else if (e.key === "ArrowDown") next = active + cols;
        else if (e.key === "ArrowUp") {
            if (active < cols) {
                e.preventDefault();
                searchRef.current?.focus();
                return;
            }
            next = active - cols;
        } else if (e.key === "Home") next = e.ctrlKey ? 0 : active - (active % cols);
        else if (e.key === "End") next = e.ctrlKey ? last : Math.min(last, active - (active % cols) + cols - 1);
        if (next === null) return;
        e.preventDefault();
        focusItem(next);
    };

    const onSearchKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === "ArrowDown") {
            e.preventDefault();
            focusItem(0);
        } else if (e.key === "Enter" && query.trim()) {
            // Enter in the search box adds the best match that is not already seated.
            const first = shown.find(pickable);
            if (first) {
                e.preventDefault();
                onPick(first.id);
            }
        } else if (e.key === "Escape" && query) {
            e.preventDefault();
            e.stopPropagation();
            setQuery("");
        }
    };

    const onSectionKeyDown = (e: React.KeyboardEvent<HTMLElement>) => {
        // A select's popup is portalled out of the section but still bubbles here through React; it closes itself.
        if (e.key !== "Escape" || e.defaultPrevented || !e.currentTarget.contains(e.target as Node)) return;
        e.preventDefault();
        onClose();
    };

    const filtersActive = query.trim() !== "" || cls !== "all" || branch !== "all" || rarity !== "all";
    const clearFilters = () => {
        setQuery("");
        setCls("all");
        setBranch("all");
        setRarity("all");
        searchRef.current?.focus();
    };

    const title = isSupport ? t("squad.picker.supportTitle") : t("squad.picker.title", { seat: seat + 1 });
    const sortLabels: Record<SortKey, string> = { rarity: t("squad.picker.sort.rarity"), name: t("squad.picker.sort.name"), cost: t("squad.picker.sort.cost"), level: t("squad.picker.sort.level") };
    const sortLabel = (s: SortKey) => sortLabels[s];

    return (
        <section aria-label={title} onKeyDown={onSectionKeyDown} className="rounded-xl border border-border bg-card">
            {/* Pinned under the site header from 640 px up, so the filters stay in reach down a long list. */}
            <div className="z-20 flex flex-col gap-3 rounded-t-xl border-border border-b bg-card/95 p-3 backdrop-blur-md supports-[backdrop-filter]:bg-card/85 sm:sticky sm:top-16 sm:p-4">
                <div className="flex items-center justify-between gap-2">
                    <h3 className="font-semibold text-[15px]">{title}</h3>
                    <Button variant="ghost" size="icon" aria-label={t("squad.picker.close")} onClick={onClose} className="pointer-coarse:size-11">
                        <X />
                    </Button>
                </div>

                <div className="flex flex-wrap items-center gap-x-3 gap-y-2">
                    <Input ref={searchRef} type="search" value={query} onChange={(e) => setQuery(e.target.value)} onKeyDown={onSearchKeyDown} placeholder={t("squad.picker.search")} aria-label={t("squad.picker.search")} className="min-w-48 flex-1 basis-60" />
                    <div className="flex items-center gap-3">
                        {showOwnedFilter && (
                            <Label className="flex min-h-9 pointer-coarse:min-h-11 cursor-pointer items-center gap-2 whitespace-nowrap text-[13px]">
                                <Switch
                                    checked={ownedOnly}
                                    onCheckedChange={(v) => {
                                        touchedOwned.current = true;
                                        setOwnedOnly(v);
                                    }}
                                />
                                {t("squad.picker.owned")}
                            </Label>
                        )}
                        <Select value={sort} onValueChange={(v) => v && setSort(v as SortKey)}>
                            <SelectTrigger size="sm" className="w-36 min-w-0" aria-label={t("squad.picker.sort")}>
                                <SelectValue>{(v: string | null) => sortLabel((v as SortKey | null) ?? "rarity")}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                {sortable.map((s) => (
                                    <SelectItem key={s} value={s}>
                                        {sortLabel(s)}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                </div>

                <div className="flex flex-wrap items-center gap-2">
                    <ToggleGroup
                        aria-label={t("squad.picker.class")}
                        value={[cls]}
                        onValueChange={(next) => {
                            setCls((next[0] as string | undefined) ?? "all");
                            setBranch("all");
                        }}
                        variant="outline"
                        className="flex-wrap"
                    >
                        <ToggleGroupItem value="all" className={cn("px-2.5 text-[13px]", TOUCH_ITEM)}>
                            {t("squad.picker.all")}
                        </ToggleGroupItem>
                        {PROFESSION_ORDER.map((p) => (
                            <Tooltip key={p}>
                                <TooltipTrigger render={<ToggleGroupItem value={p} aria-label={professionLabel({ profession: p })} className={cn("px-1.5", TOUCH_ITEM)} />}>
                                    <ClassIcon profession={p} size={20} className="size-5!" />
                                </TooltipTrigger>
                                <TooltipPopup>{professionLabel({ profession: p })}</TooltipPopup>
                            </Tooltip>
                        ))}
                    </ToggleGroup>
                    <ToggleGroup aria-label={t("squad.picker.rarity")} value={[rarity]} onValueChange={(next) => setRarity((next[0] as string | undefined) ?? "all")} variant="outline" className="flex-wrap">
                        <ToggleGroupItem value="all" className={cn("px-2.5 text-[13px]", TOUCH_ITEM)}>
                            {t("squad.picker.all")}
                        </ToggleGroupItem>
                        {rarities.map((r) => (
                            <ToggleGroupItem key={r} value={String(r)} className={cn("px-2 font-mono text-[13px]", TOUCH_ITEM)}>
                                {t("squad.picker.stars", { rarity: r })}
                            </ToggleGroupItem>
                        ))}
                    </ToggleGroup>
                    {branches.length > 1 && (
                        <Select value={branch} onValueChange={(v) => v && setBranch(v)}>
                            <SelectTrigger size="sm" className="w-auto min-w-44" aria-label={t("squad.picker.branch")}>
                                <SelectValue>{(v: string | null) => (v && v !== "all" ? (branches.find((b) => b.id === v)?.label ?? v) : t("squad.picker.allBranches"))}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="all">{t("squad.picker.allBranches")}</SelectItem>
                                {branches.map((b) => (
                                    <SelectItem key={b.id} value={b.id}>
                                        <span className="flex items-center gap-2">
                                            <SubProfessionIcon subProfession={b.id} size={16} className="size-4 shrink-0" />
                                            {b.label}
                                        </span>
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    )}
                </div>
            </div>

            <div className="p-3 sm:p-4">
                {operators.length === 0 ? (
                    <p className="py-8 text-center text-[13px] text-muted-foreground">{t("squad.picker.loading")}</p>
                ) : shown.length === 0 ? (
                    <div className="flex flex-col items-center gap-3 py-8">
                        <p className="text-center text-[13px] text-muted-foreground">{t("squad.picker.empty")}</p>
                        {filtersActive && (
                            <Button variant="outline" size="sm" onClick={clearFilters} className="pointer-coarse:h-11">
                                {t("squad.picker.clearFilters")}
                            </Button>
                        )}
                    </div>
                ) : (
                    <>
                        <div className="mb-2.5 flex items-center justify-between gap-3">
                            <p className="font-mono text-[12px] text-muted-foreground tabular-nums">{t("squad.picker.count", { count: shown.length })}</p>
                            <p aria-hidden="true" className="hidden items-center gap-1.5 text-[12px] text-muted-foreground sm:pointer-fine:flex">
                                <Kbd>←</Kbd>
                                <Kbd>→</Kbd>
                                <Kbd>↑</Kbd>
                                <Kbd>↓</Kbd>
                                <span className="me-1.5">{t("squad.picker.hint.move")}</span>
                                <Kbd>Enter</Kbd>
                                <span className="me-1.5">{t("squad.picker.hint.pick")}</span>
                                <Kbd>Esc</Kbd>
                                <span>{t("squad.picker.hint.close")}</span>
                            </p>
                        </div>
                        <ul ref={gridRef} aria-label={title} onKeyDown={onGridKeyDown} className="grid grid-cols-[repeat(auto-fill,minmax(9.5rem,1fr))] gap-2 sm:grid-cols-[repeat(auto-fill,minmax(13.5rem,1fr))]">
                            {shown.map((op, i) => (
                                <li key={op.id} className="flex">
                                    <PickerTile op={op} build={roster.get(op.id)} showOwnership={hasRoster && !isSupport} seatedAt={seated.get(op.id)} tabIndex={i === active ? 0 : -1} onFocus={() => setActive(i)} onPick={onPick} t={t} />
                                </li>
                            ))}
                        </ul>
                    </>
                )}
            </div>
        </section>
    );
}

interface IPickerTileProps {
    op: IOperatorIndexEntry;
    build: IRosterBuild | undefined;
    /** Whether a missing build reads as "Not owned": only against a roster, never in the support picker. */
    showOwnership: boolean;
    seatedAt: number | undefined;
    tabIndex: number;
    onFocus: () => void;
    onPick: (id: string) => void;
    t: PickerT;
}

const PickerTile = React.memo(function PickerTile({ op, build, showOwnership, seatedAt, tabIndex, onFocus, onPick, t }: IPickerTileProps): React.ReactElement {
    const seated = seatedAt !== undefined;
    const current = isCurrentForm(op.id, build);
    const branch = archetypeLabel(op);
    const color = rarityStarColor(op.rarity);
    return (
        <button
            type="button"
            data-picker-item=""
            // Seated tiles stay focusable so the arrow keys walk through them; `aria-disabled` keeps them in the tab order.
            aria-disabled={seated || undefined}
            tabIndex={tabIndex}
            onFocus={onFocus}
            onClick={() => {
                if (!seated) onPick(op.id);
            }}
            aria-label={t("squad.picker.pick", { name: op.name, branch })}
            className={cn(
                "group flex w-full min-w-0 cursor-pointer items-start gap-2 rounded-lg border border-border bg-background/40 p-1.5 text-left outline-none transition-colors duration-150 motion-reduce:transition-none sm:gap-2.5 sm:p-2",
                "hover:border-primary/60 hover:bg-accent/40 focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-ring",
                current && "border-primary/45 bg-primary/[0.06]",
                seated && "cursor-default opacity-55 hover:border-border hover:bg-background/40",
            )}
        >
            <span className="relative block size-11 shrink-0 overflow-hidden rounded-md bg-muted sm:size-14" style={{ boxShadow: `inset 0 -3px 0 ${color}` }}>
                <img alt="" src={getAvatarById(op.id)} className="size-full object-cover" loading="lazy" decoding="async" />
                {seated && <span className="absolute inset-x-0 bottom-[3px] bg-primary text-center font-semibold text-[10px] text-primary-foreground leading-4">{seatedAt === SUPPORT_SEAT ? t("squad.picker.seatedSupport") : t("squad.picker.seated", { seat: (seatedAt ?? 0) + 1 })}</span>}
            </span>
            <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                <span className="flex flex-wrap items-center gap-x-1.5 gap-y-0.5">
                    <span className="break-words font-semibold text-[14px] text-foreground leading-snug">{op.name}</span>
                    {current && <span className="rounded-sm bg-primary/15 px-1 font-semibold text-[10.5px] text-primary uppercase leading-4 tracking-wide">{t("squad.picker.currentForm")}</span>}
                </span>
                <span className="flex items-center gap-1 text-[12px] text-muted-foreground leading-tight">
                    <ClassIcon profession={op.profession} size={14} className="size-3.5! shrink-0" />
                    <span className="min-w-0">{branch}</span>
                </span>
                <span className="flex flex-wrap items-center gap-x-2 gap-y-0.5 pt-0.5 text-[11.5px] leading-4">
                    <span role="img" aria-label={t("squad.picker.stars", { rarity: op.rarity })} className="font-semibold text-[10px] tracking-[-1px]" style={{ color }}>
                        {"★".repeat(op.rarity)}
                    </span>
                    {op.stats.cost > 0 && (
                        <span title={t("squad.picker.costTitle")} className="font-mono text-muted-foreground tabular-nums">
                            {t("squad.picker.cost", { cost: op.stats.cost })}
                        </span>
                    )}
                </span>
                {build ? <BuildLine op={op} build={build} t={t} /> : showOwnership && <span className="text-[11.5px] text-muted-foreground/80 italic leading-4">{t("squad.picker.notOwned")}</span>}
            </span>
        </button>
    );
});

/** The owned build in one line: promotion and level, potential, the selected skill and its mastery, the equipped module and its rank. */
function BuildLine({ op, build, t }: { op: IOperatorIndexEntry; build: IRosterBuild; t: PickerT }): React.ReactElement {
    const skill = build.default_skill !== null && build.default_skill >= 0 && build.default_skill < op.skillCount ? build.default_skill : null;
    const mastery = skill === null ? 0 : (build.masteries.find((m) => m.index === skill)?.mastery ?? 0);
    const equipped = build.current_equip ? advancedModules(op).find((m) => m.uniEquipId === build.current_equip) : undefined;
    const moduleRank = equipped ? (rosterModuleRanks(op, build).get(equipped.uniEquipId) ?? 0) : 0;
    const potential = build.potential + 1;
    return (
        <span className="flex flex-wrap items-center gap-x-2 gap-y-0.5 font-mono text-[11.5px] text-muted-foreground tabular-nums leading-4">
            <span className="flex items-center gap-0.5 text-foreground/90">
                <img alt={t("squad.picker.elite", { phase: build.elite })} src={eliteIcon(build.elite)} className="icon-theme-aware size-3.5 object-contain" />
                {t("squad.picker.level", { level: build.level })}
            </span>
            {op.potentialRankCount > 0 && <img alt={t("squad.picker.potential", { rank: potential })} title={t("squad.picker.potential", { rank: potential })} src={potentialIcon(build.potential)} className="size-3.5 object-contain" />}
            {skill !== null && (
                <span className="flex items-center gap-0.5" title={t("squad.picker.skillTitle", { skill: skill + 1, level: build.skill_level })}>
                    {t("squad.picker.skill", { skill: skill + 1 })}
                    {mastery > 0 && <img alt={t("squad.picker.mastery", { mastery })} src={specializedIcon(mastery)} className="size-3.5 object-contain" />}
                </span>
            )}
            {equipped && moduleRank > 0 && <span title={t("squad.picker.moduleTitle", { type: moduleTypeLabel(equipped), rank: moduleRank })}>{t("squad.picker.module", { type: moduleTypeLabel(equipped), rank: moduleRank })}</span>}
        </span>
    );
}
