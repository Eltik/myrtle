import { useQuery } from "@tanstack/react-query";
import { Users } from "lucide-react";
import * as React from "react";
import { skillRankShort } from "#/components/tools/shared/skill";
import type { messages as skillMessages } from "#/components/tools/shared/skill.messages";
import { Button } from "#/components/ui/button";
import { Dialog, DialogDescription, DialogHeader, DialogPanel, DialogPopup, DialogTitle, DialogTrigger } from "#/components/ui/dialog";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Spinner } from "#/components/ui/spinner";
import { Switch } from "#/components/ui/switch";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { dpsSupportersQueryOptions } from "#/lib/api/dps";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { searchAndRank } from "#/lib/search/fuzzy";
import { formatProfession } from "#/lib/utils";
import type { SupporterEntry } from "#/types/generated/SupporterEntry";
import type { SupporterInvestment } from "#/types/generated/SupporterInvestment";
import type { SupporterTarget } from "#/types/generated/SupporterTarget";
import { formatZoneValue, STACKING_LABEL_KEYS, SUPPORTER_ZONE_KEYS, type ZoneMessageKey, zoneField } from "../zones";
import type { messages } from "./ZonePanel.messages";
import type { messages as zoneMessages } from "./zones.messages";

type PickerT = TypedT<typeof messages & typeof zoneMessages & typeof skillMessages>;

export const TARGET_LABEL_KEYS: Readonly<Record<SupporterTarget, ZoneMessageKey>> = {
    allAllies: "dps.supporter.target.allAllies",
    alliesInRange: "dps.supporter.target.alliesInRange",
    oneAlly: "dps.supporter.target.oneAlly",
    enemies: "dps.supporter.target.enemies",
};

/** "S2" / "T1" style tag for where a supporter's effect comes from. */
export function useSourceLabel(): (entry: Pick<SupporterEntry, "sourceKind" | "sourceIndex">) => string {
    const t: PickerT = useT("tools");
    return React.useCallback((entry) => (entry.sourceKind === "talent" ? t("dps.supporters.talent", { index: entry.sourceIndex }) : t("dps.supporters.skill", { index: entry.sourceIndex })), [t]);
}

interface ISupporterPickerProps {
    selected: readonly string[];
    investment: SupporterInvestment;
    onToggle: (id: string, on: boolean) => void;
    onInvestment: (patch: Partial<SupporterInvestment>) => void;
}

const RANKS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] as const;

/** The supporter preset dialog: investment, search, and one switch per resolved entry. */
export function SupporterPicker({ selected, investment, onToggle, onInvestment }: ISupporterPickerProps): React.ReactElement {
    const t: PickerT = useT("tools");
    const [open, setOpen] = React.useState(false);
    const [query, setQuery] = React.useState("");
    const { data, isPending, error } = useQuery({ ...dpsSupportersQueryOptions(investment), enabled: open });
    const sourceLabel = useSourceLabel();
    const entries = React.useMemo(() => {
        const all = data?.entries ?? [];
        if (!query.trim()) return all;
        return searchAndRank(query, all, (e) => ({ name: e.name, extra: `${e.sourceName} ${sourceLabel(e)} ${e.contributions.map((c) => c.zone).join(" ")}` })).map((r) => r.item);
    }, [data, query, sourceLabel]);
    const rankLabel = (r: number) => skillRankShort(r, t);

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger
                render={(p) => (
                    <Button {...p} variant="outline" size="sm">
                        <Users />
                        {t("dps.supporters.add")}
                        {selected.length > 0 && <span className="rounded bg-primary/15 px-1.5 font-mono text-[10px] text-primary">{selected.length}</span>}
                    </Button>
                )}
            />
            <DialogPopup className="sm:max-w-2xl">
                <DialogHeader>
                    <DialogTitle>{t("dps.supporters.title")}</DialogTitle>
                    <DialogDescription>{t("dps.supporters.desc")}</DialogDescription>
                </DialogHeader>
                <DialogPanel className="space-y-3">
                    <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                        <InvestmentSelect label={t("dps.supporters.promotion")} value={investment.promotion} options={[0, 1, 2]} format={(v) => `E${v}`} onChange={(promotion) => onInvestment({ promotion })} />
                        <InvestmentSelect label={t("dps.supporters.skillLevel")} value={investment.skillLevel} options={RANKS} format={rankLabel} onChange={(skillLevel) => onInvestment({ skillLevel })} />
                        <InvestmentSelect label={t("dps.supporters.potential")} value={investment.potential} options={[1, 2, 3, 4, 5, 6]} format={(v) => t("dps.supporters.potentialValue", { rank: v })} onChange={(potential) => onInvestment({ potential })} />
                        <InvestmentSelect label={t("dps.supporters.moduleLevel")} value={investment.moduleLevel} options={[0, 1, 2, 3]} format={(v) => (v === 0 ? t("dps.supporters.moduleNone") : `L${v}`)} onChange={(moduleLevel) => onInvestment({ moduleLevel })} />
                    </div>
                    <Input type="search" size="sm" value={query} onChange={(e) => setQuery(e.target.value)} placeholder={t("dps.supporters.search")} aria-label={t("dps.supporters.search")} />
                    {isPending ? (
                        <div className="flex items-center gap-2 py-6 text-[12px] text-muted-foreground">
                            <Spinner className="size-3.5" />
                            {t("dps.supporters.loading")}
                        </div>
                    ) : error ? (
                        <p className="py-6 text-[12px] text-destructive">{t("dps.supporters.failed", { error: error.message })}</p>
                    ) : entries.length === 0 ? (
                        <p className="py-6 text-[12px] text-muted-foreground">{t("dps.supporters.none")}</p>
                    ) : (
                        <ul className="divide-y divide-border rounded-lg border border-border">
                            {entries.map((entry) => (
                                <SupporterRow key={entry.id} entry={entry} on={selected.includes(entry.id)} onToggle={(on) => onToggle(entry.id, on)} />
                            ))}
                        </ul>
                    )}
                    {data && data.unresolved.length > 0 && <p className="text-[11px] text-muted-foreground">{t("dps.supporters.unresolved", { count: data.unresolved.length })}</p>}
                </DialogPanel>
            </DialogPopup>
        </Dialog>
    );
}

function SupporterRow({ entry, on, onToggle }: { entry: SupporterEntry; on: boolean; onToggle: (on: boolean) => void }): React.ReactElement {
    const t: PickerT = useT("tools");
    const locale = useLocale();
    const sourceLabel = useSourceLabel();
    const classes = entry.scope.professions.map((p) => formatProfession(p)).join(", ");
    return (
        <li className="flex items-start gap-3 px-3 py-2.5">
            <span aria-hidden="true" className="inline-flex size-9 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted font-semibold text-[10px]">
                <OperatorAvatar charId={entry.charId} name={entry.name} />
            </span>
            <div className="min-w-0 flex-1 space-y-1">
                <div className="flex flex-wrap items-baseline gap-x-1.5">
                    <span className="font-semibold text-[13px]">{entry.name}</span>
                    <span className="text-[11.5px] text-muted-foreground">
                        {sourceLabel(entry)} · {entry.sourceName}
                    </span>
                </div>
                <div className="flex flex-wrap gap-1">
                    {entry.contributions.map((c) => {
                        const field = zoneField(SUPPORTER_ZONE_KEYS[c.zone]);
                        return (
                            <span key={`${c.zone}-${c.key}`} className="rounded bg-muted px-1.5 py-0.5 text-[11px]">
                                <span className="text-muted-foreground">{t(field.labelKey)}</span> <span className="font-mono tabular-nums">{formatZoneValue(field.unit, c.value, locale)}</span>
                                <span className="text-muted-foreground"> · {t(STACKING_LABEL_KEYS[c.stacking === "max" ? "max" : "additive"])}</span>
                                {c.transform === "timesAtk" && entry.supporterAtk !== undefined && <span className="text-muted-foreground"> · {t("dps.supporters.inspireFrom", { atk: Math.round(entry.supporterAtk) })}</span>}
                            </span>
                        );
                    })}
                </div>
                <div className="flex flex-wrap items-center gap-x-2 gap-y-0.5 text-[11px] text-muted-foreground">
                    <span>{t(TARGET_LABEL_KEYS[entry.scope.target])}</span>
                    {classes && <span>{classes}</span>}
                    {entry.duration !== undefined && entry.duration > 0 && (
                        <Tooltip>
                            <TooltipTrigger render={<span className="cursor-help underline decoration-dotted underline-offset-2" />}>{t("dps.supporters.duration", { seconds: Math.round(entry.duration * 10) / 10 })}</TooltipTrigger>
                            <TooltipPopup className="max-w-60">{t("dps.supporters.durationHint", { seconds: Math.round(entry.duration * 10) / 10 })}</TooltipPopup>
                        </Tooltip>
                    )}
                    {entry.scope.condition && <span className="italic">{entry.scope.condition}</span>}
                </div>
            </div>
            <Switch checked={on} onCheckedChange={onToggle} aria-label={t("dps.supporters.toggle", { name: `${entry.name} ${sourceLabel(entry)}` })} />
        </li>
    );
}

function InvestmentSelect({ label, value, options, format, onChange }: { label: string; value: number; options: readonly number[]; format: (v: number) => string; onChange: (v: number) => void }): React.ReactElement {
    return (
        <div className="space-y-1">
            <Label className="block font-medium text-[11px] text-muted-foreground leading-none">{label}</Label>
            <Select value={String(value)} onValueChange={(v) => v !== null && onChange(Number(v))}>
                <SelectTrigger size="sm" aria-label={label}>
                    <SelectValue>{() => format(value)}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                    {options.map((o) => (
                        <SelectItem key={o} value={String(o)}>
                            {format(o)}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>
        </div>
    );
}
