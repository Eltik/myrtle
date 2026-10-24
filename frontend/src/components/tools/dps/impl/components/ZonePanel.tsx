import { HelpCircle, X } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { Card, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { Spinner } from "#/components/ui/spinner";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, formatProfession } from "#/lib/utils";
import type { SupporterInvestment } from "#/types/generated/SupporterInvestment";
import type { ITeamBuffs } from "../types";
import type { ITeamBuffsContext } from "../useTeamBuffs";
import { countSetZones, formatZoneValue, type IZoneConfig, resolveZone, STACKING_LABEL_KEYS, ZONE_FIELDS, ZONE_HELPER_ROWS, zoneField } from "../zones";
import { SupporterPicker, useSourceLabel } from "./SupporterPicker";
import { ZoneFields } from "./ZoneFields";
import type { messages } from "./ZonePanel.messages";
import type { messages as zoneMessages } from "./zones.messages";

type PanelT = TypedT<typeof messages & typeof zoneMessages>;

interface ITeamBuffsPanelProps {
    team: ITeamBuffs;
    ctx: ITeamBuffsContext;
    /** Bumped once after `localStorage` hydration so the uncontrolled inputs remount with the stored values. */
    hydrationToken: number;
    onChangeZones: (patch: Partial<IZoneConfig>) => void;
    onToggleSupporter: (id: string, on: boolean) => void;
    onInvestment: (patch: Partial<SupporterInvestment>) => void;
    onClear: () => void;
}

/** The page-wide buff panel: zone inputs, supporter presets, and how their sources stack. */
export function TeamBuffsPanel({ team, ctx, hydrationToken, onChangeZones, onToggleSupporter, onInvestment, onClear }: ITeamBuffsPanelProps): React.ReactElement {
    const t: PanelT = useT("tools");
    const [clearCount, setClearCount] = React.useState(0);
    const anySet = countSetZones(team.zones) > 0 || team.supporters.length > 0;
    return (
        <Card>
            <CardHeader>
                <div className="flex items-center justify-between gap-2">
                    <CardTitle className="text-[15px]">{t("dps.team.title")}</CardTitle>
                    <div className="flex items-center gap-1">
                        <ZoneHelper />
                        {anySet && (
                            <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => {
                                    onClear();
                                    setClearCount((c) => c + 1);
                                }}
                            >
                                {t("dps.team.clear")}
                            </Button>
                        )}
                    </div>
                </div>
                <p className="text-[12px] text-muted-foreground">{t("dps.team.desc")}</p>
            </CardHeader>
            <CardPanel className="space-y-3 pt-0">
                <div className="flex flex-wrap items-center gap-1.5">
                    <SupporterPicker selected={team.supporters} investment={team.investment} onToggle={onToggleSupporter} onInvestment={onInvestment} />
                    <ActiveSupporters team={team} ctx={ctx} onToggle={onToggleSupporter} />
                </div>
                <ZoneFields zones={team.zones} onChange={onChangeZones} resetKey={`${hydrationToken}-${clearCount}`} />
                <StackingSummary team={team} ctx={ctx} />
            </CardPanel>
        </Card>
    );
}

function ActiveSupporters({ team, ctx, onToggle }: { team: ITeamBuffs; ctx: ITeamBuffsContext; onToggle: (id: string, on: boolean) => void }): React.ReactElement | null {
    const t: PanelT = useT("tools");
    const sourceLabel = useSourceLabel();
    if (team.supporters.length === 0) return null;
    if (ctx.waiting)
        return (
            <span className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
                <Spinner className="size-3" />
                {t("dps.supporters.waiting")}
            </span>
        );
    const byId = new Map((ctx.entries ?? []).map((e) => [e.id, e]));
    return (
        <>
            {ctx.error && <p className="w-full text-[11px] text-destructive">{t("dps.supporters.blocked", { error: ctx.error.message })}</p>}
            {team.supporters.map((id) => {
                const entry = byId.get(id);
                const name = entry ? `${entry.name} ${sourceLabel(entry)}` : id;
                return (
                    <span key={id} className="inline-flex h-7 items-center gap-1 rounded-md border border-primary/40 bg-primary/10 pr-0.5 pl-2 font-medium text-[11px]">
                        {name}
                        <Button aria-label={t("dps.supporters.remove", { name })} variant="ghost" size="icon-xs" onClick={() => onToggle(id, false)}>
                            <X />
                        </Button>
                    </span>
                );
            })}
        </>
    );
}

/**
 * Per zone that a supporter feeds: every source, which ones apply, and what reaches the
 * engine. Shows the team layer only; a card's own zones join on top per card.
 */
function StackingSummary({ team, ctx }: { team: ITeamBuffs; ctx: ITeamBuffsContext }): React.ReactElement | null {
    const t: PanelT = useT("tools");
    const locale = useLocale();
    const sourceLabel = useSourceLabel();
    const byId = new Map((ctx.entries ?? []).map((e) => [e.id, e]));
    const zones = ZONE_FIELDS.filter((f) => ctx.sources.some((s) => s.zone === f.key));
    if (zones.length === 0) return null;
    return (
        <div className="space-y-1.5 rounded-md border border-border bg-muted/30 p-2">
            <h3 className="font-medium text-[11px] text-muted-foreground">{t("dps.stacking.title")}</h3>
            {zones.map((f) => {
                const r = resolveZone(f.key, [{ origin: "team", zones: team.zones }], ctx.sources);
                return (
                    <div key={f.key} className="text-[11.5px]">
                        <div className="flex items-baseline justify-between gap-2">
                            <span className="font-medium">
                                {t(f.labelKey)} <span className="font-normal text-muted-foreground">({t(STACKING_LABEL_KEYS[r.stacking])})</span>
                            </span>
                            <span className="font-mono text-[11px] tabular-nums">{t("dps.stacking.applied", { value: formatZoneValue(f.unit, r.total, locale) })}</span>
                        </div>
                        <div className="mt-0.5 flex flex-wrap gap-1">
                            {r.sources.map((s) => {
                                const entry = byId.get(s.origin);
                                const name = s.origin === "team" ? t("dps.stacking.typed") : entry ? `${entry.name} ${sourceLabel(entry)}` : s.origin;
                                const scoped = entry && entry.scope.professions.length > 0 ? entry.scope.professions.map((p) => formatProfession(p)).join(", ") : null;
                                const note = !s.applied ? t("dps.stacking.ignored") : scoped ? t("dps.stacking.outOfScope", { classes: scoped }) : null;
                                return <SourceChip key={s.id} name={name} value={formatZoneValue(zoneField(f.key).unit, s.value, locale)} applied={s.applied} note={note} />;
                            })}
                        </div>
                    </div>
                );
            })}
        </div>
    );
}

function SourceChip({ name, value, applied, note }: { name: string; value: string; applied: boolean; note: string | null }): React.ReactElement {
    const chip = (
        <span className={cn("inline-flex items-center gap-1 rounded px-1.5 py-0.5", applied ? "bg-primary/12 text-foreground" : "bg-background text-muted-foreground line-through")}>
            {name}
            <span className="font-mono tabular-nums">{value}</span>
        </span>
    );
    if (!note) return chip;
    return (
        <Tooltip>
            <TooltipTrigger render={<span className="cursor-help" />}>{chip}</TooltipTrigger>
            <TooltipPopup className="max-w-60">{note}</TooltipPopup>
        </Tooltip>
    );
}

/** "Which buff is this?": game phrasings mapped to the field they belong in. */
function ZoneHelper(): React.ReactElement {
    const t: PanelT = useT("tools");
    return (
        <Popover>
            <PopoverTrigger
                render={(p) => (
                    <Button {...p} variant="ghost" size="sm" className="text-muted-foreground hover:text-foreground">
                        <HelpCircle />
                        {t("dps.helper.open")}
                    </Button>
                )}
            />
            <PopoverPopup className="w-[min(420px,calc(100vw-2rem))]">
                <p className="mb-2 text-[12px] text-muted-foreground">{t("dps.helper.intro")}</p>
                <dl className="max-h-[min(70vh,560px)] space-y-2.5 overflow-y-auto text-[12px]">
                    {ZONE_HELPER_ROWS.map((row) => (
                        <div key={row.wordingKey}>
                            <dt className="text-foreground">{t(row.wordingKey)}</dt>
                            <dd className="mt-0.5 flex flex-wrap gap-1">
                                {row.zones.map((z) => (
                                    <span key={z} className="rounded bg-primary/10 px-1.5 py-0.5 font-medium text-[11px] text-primary">
                                        {t(zoneField(z).labelKey)}
                                    </span>
                                ))}
                            </dd>
                            <dd className="mt-0.5 text-[11.5px] text-muted-foreground">{t(row.noteKey)}</dd>
                        </div>
                    ))}
                </dl>
            </PopoverPopup>
        </Popover>
    );
}
