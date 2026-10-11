import { X } from "lucide-react";
import * as React from "react";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { PROFESSION_ORDER } from "#/components/operators/list/impl/constants";
import { eliteIcon } from "#/components/tools/shared/icons";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { Switch } from "#/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { searchAndRank } from "#/lib/search/fuzzy";
import { cn, formatProfession, getAvatarById, rarityStarColor } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import { type IRosterBuild, SUPPORT_SEAT } from "../model";
import type { messages } from "./OperatorPicker.messages";

/** A filter chip is 28 px tall from 640 px up; on a touch screen it takes the full 44 px. */
const TOUCH_ITEM = "pointer-coarse:h-11 pointer-coarse:min-w-11";

type PickerT = TypedT<typeof messages>;

export interface IOperatorPickerProps {
    /** 0-11, or `SUPPORT_SEAT` for the support picker. */
    seat: number;
    operators: readonly IOperatorIndexEntry[];
    roster: ReadonlyMap<string, IRosterBuild>;
    /** `char_id` -> seat it already sits in (`SUPPORT_SEAT` for the support). */
    seated: ReadonlyMap<string, number>;
    onPick: (charId: string) => void;
    onClose: () => void;
}

export function OperatorPicker({ seat, operators, roster, seated, onPick, onClose }: IOperatorPickerProps): React.ReactElement {
    const t: PickerT = useT("tools");
    const { collator } = useFormatters();
    const isSupport = seat === SUPPORT_SEAT;
    const hasRoster = roster.size > 0;
    const [query, setQuery] = React.useState("");
    const [ownedOnly, setOwnedOnly] = React.useState(hasRoster);
    const [cls, setCls] = React.useState<string>("all");
    const [rarity, setRarity] = React.useState<string>("all");
    const searchRef = React.useRef<HTMLInputElement>(null);

    // The roster can arrive after the picker opens; follow it until the player touches the switch.
    const touchedOwned = React.useRef(false);
    React.useEffect(() => {
        if (!touchedOwned.current) setOwnedOnly(hasRoster);
    }, [hasRoster]);

    React.useEffect(() => {
        searchRef.current?.focus({ preventScroll: true });
    }, []);

    // The star filter offers the rarities the index has, highest first.
    const rarities = React.useMemo(() => [...new Set(operators.map((op) => op.rarity))].sort((a, b) => b - a), [operators]);

    const pool = React.useMemo(() => {
        const professions = new Set<string>(PROFESSION_ORDER);
        return operators.filter((op) => {
            if (!professions.has(op.profession)) return false;
            const owned = roster.has(op.id);
            if (op.isNotObtainable && !owned) return false;
            if (!isSupport && ownedOnly && hasRoster && !owned) return false;
            if (cls !== "all" && op.profession !== cls) return false;
            if (rarity !== "all" && op.rarity !== Number(rarity)) return false;
            return true;
        });
    }, [operators, roster, isSupport, ownedOnly, hasRoster, cls, rarity]);

    const shown = React.useMemo(() => {
        if (query.trim()) return searchAndRank(query, pool, (op) => ({ name: op.name, aliases: op.appellation ? [op.appellation] : [] })).map((r) => r.item);
        return [...pool].sort((a, b) => b.rarity - a.rarity || collator.compare(a.name, b.name));
    }, [pool, query, collator]);

    const title = isSupport ? t("squad.picker.supportTitle") : t("squad.picker.title", { seat: seat + 1 });

    return (
        <section aria-label={title} className="rounded-xl border border-border bg-card p-4">
            <div className="mb-3 flex items-center justify-between gap-2">
                <h3 className="font-semibold text-[15px]">{title}</h3>
                <Button variant="ghost" size="icon-sm" aria-label={t("squad.picker.close")} onClick={onClose}>
                    <X />
                </Button>
            </div>

            <div className="flex flex-col gap-3">
                <div className="flex flex-wrap items-center gap-3">
                    <Input ref={searchRef} type="search" value={query} onChange={(e) => setQuery(e.target.value)} placeholder={t("squad.picker.search")} aria-label={t("squad.picker.search")} className="min-w-48 flex-1" />
                    {!isSupport && hasRoster && (
                        <Label className="flex cursor-pointer items-center gap-2 text-[13px]">
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
                </div>
                <div className="flex flex-wrap items-center gap-2">
                    <ToggleGroup aria-label={t("squad.picker.class")} value={[cls]} onValueChange={(next) => setCls((next[0] as string | undefined) ?? "all")} variant="outline" size="sm" className="flex-wrap">
                        <ToggleGroupItem value="all" className={cn("px-2 text-[12px]", TOUCH_ITEM)}>
                            {t("squad.picker.all")}
                        </ToggleGroupItem>
                        {PROFESSION_ORDER.map((p) => (
                            <ToggleGroupItem key={p} value={p} aria-label={formatProfession(p)} title={formatProfession(p)} className={cn("px-1.5", TOUCH_ITEM)}>
                                <ClassIcon profession={p} size={18} className="size-4.5!" />
                            </ToggleGroupItem>
                        ))}
                    </ToggleGroup>
                    <ToggleGroup aria-label={t("squad.picker.rarity")} value={[rarity]} onValueChange={(next) => setRarity((next[0] as string | undefined) ?? "all")} variant="outline" size="sm" className="flex-wrap">
                        <ToggleGroupItem value="all" className={cn("px-2 text-[12px]", TOUCH_ITEM)}>
                            {t("squad.picker.all")}
                        </ToggleGroupItem>
                        {rarities.map((r) => (
                            <ToggleGroupItem key={r} value={String(r)} className={cn("px-1.5 font-mono text-[12px]", TOUCH_ITEM)}>
                                {t("squad.picker.stars", { rarity: r })}
                            </ToggleGroupItem>
                        ))}
                    </ToggleGroup>
                </div>

                {operators.length === 0 ? (
                    <p className="py-6 text-center text-[13px] text-muted-foreground">{t("squad.picker.loading")}</p>
                ) : shown.length === 0 ? (
                    <p className="py-6 text-center text-[13px] text-muted-foreground">{t("squad.picker.empty")}</p>
                ) : (
                    <>
                        <p className="font-mono text-[11px] text-muted-foreground">{t("squad.picker.count", { count: shown.length })}</p>
                        <ul className="grid max-h-[26rem] grid-cols-[repeat(auto-fill,minmax(4.5rem,1fr))] gap-2 overflow-y-auto pr-1">
                            {shown.map((op) => (
                                <li key={op.id}>
                                    <PickerTile op={op} roster={roster.get(op.id)} seatedAt={seated.get(op.id)} onPick={onPick} t={t} />
                                </li>
                            ))}
                        </ul>
                    </>
                )}
            </div>
        </section>
    );
}

function PickerTile({ op, roster, seatedAt, onPick, t }: { op: IOperatorIndexEntry; roster: IRosterBuild | undefined; seatedAt: number | undefined; onPick: (id: string) => void; t: PickerT }): React.ReactElement {
    const seated = seatedAt !== undefined;
    return (
        <button
            type="button"
            disabled={seated}
            onClick={() => onPick(op.id)}
            aria-label={t("squad.picker.pick", { name: op.name })}
            title={op.name}
            className={cn("group flex w-full cursor-pointer flex-col items-stretch gap-1 rounded-lg border border-border bg-background/40 p-1 text-left outline-none transition-colors hover:border-primary/60 focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-default", seated && "opacity-45")}
        >
            <span className="relative block aspect-square w-full overflow-hidden rounded-md bg-muted" style={{ boxShadow: `inset 0 -2px 0 ${rarityStarColor(op.rarity)}` }}>
                <img alt="" src={getAvatarById(op.id)} className="size-full object-cover" loading="lazy" decoding="async" />
                <span className="absolute top-0.5 left-0.5 rounded bg-background/80 p-px">
                    <ClassIcon profession={op.profession} size={14} className="size-3.5!" />
                </span>
                {seated && <span className="absolute inset-x-0 bottom-0.5 mx-auto w-fit rounded bg-primary px-1 font-semibold text-[9.5px] text-primary-foreground">{seatedAt === SUPPORT_SEAT ? t("squad.picker.seatedSupport") : t("squad.picker.seated", { seat: (seatedAt ?? 0) + 1 })}</span>}
            </span>
            <span className="truncate font-medium text-[11px] leading-tight">{op.name}</span>
            <span className="flex h-3.5 items-center gap-1">
                <span className="font-semibold text-[9px] leading-none tracking-[-1px]" style={{ color: rarityStarColor(op.rarity) }}>
                    {"★".repeat(op.rarity)}
                </span>
                {roster && (
                    <span className="ml-auto flex items-center gap-0.5 font-mono text-[9.5px] text-muted-foreground tabular-nums">
                        <img alt={`E${roster.elite}`} src={eliteIcon(roster.elite)} className="icon-theme-aware size-3 object-contain" />
                        {t("squad.picker.level", { level: roster.level })}
                    </span>
                )}
            </span>
        </button>
    );
}
