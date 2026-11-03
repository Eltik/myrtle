import { ArrowDown, ArrowUp, ChevronDown, ChevronUp, Copy, Eye, EyeOff, X } from "lucide-react";
import * as React from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardHeader, CardPanel } from "#/components/ui/card";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Label } from "#/components/ui/label";
import { LoadFailed } from "#/components/ui/load-failed";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Slider } from "#/components/ui/slider";
import { Switch } from "#/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, RARITY_LABELS, rarityInk, rarityToNumber } from "#/lib/utils";
import { conditionalFlagFor, conditionalLabel, conditionalSwitchOn } from "./conditionals";
import type { messages as conditionalMessages } from "./conditionals.messages";
import type { messages } from "./InstanceCard.messages";
import { eliteIcon, moduleIconURL, potentialIcon, skillIconURL, specializedIcon } from "./icons";
import { conditionalKey } from "./instance";
import { clampRankToElite, isMasteryRank, maxSkillRankForElite, rankLockReason, skillRankLabel } from "./skill";
import type { messages as skillMessages } from "./skill.messages";
import type { IBuildConfig, ICalcConditionals, IInstance } from "./types";
import { useOperatorDetail } from "./useOperatorDetail";
import type { messages as detailMessages } from "./useOperatorDetail.messages";

/** Own chrome plus the labels `skill.ts` derives. */
type CardT = TypedT<typeof messages & typeof skillMessages & typeof detailMessages & typeof conditionalMessages>;

interface IInstanceCardProps {
    inst: IInstance;
    index: number;
    isFirst: boolean;
    isLast: boolean;
    onUpdate: (patch: Partial<IBuildConfig>) => void;
    onToggleConditional: (key: keyof ICalcConditionals, value: boolean) => void;
    onToggleVisibility: () => void;
    onToggleCollapsed: () => void;
    onMoveUp: () => void;
    onMoveDown: () => void;
    onDuplicate: () => void;
    onRemove: () => void;
    /** When provided, renders a collapsible section for this card's own buffs (DPS: its zone editor). */
    buffsSection?: React.ReactNode;
    /** Fields set in that section, shown on its heading while collapsed. */
    buffsSetCount?: number;
}

const SKILL_LEVELS = [1, 2, 3, 4, 5, 6, 7] as const;
const MASTERY_RANKS = [8, 9, 10] as const;

/** Build pickers are kit toggles drawn as separate bordered tiles on the card surface, not a joined segment. */
const CHIP = "border-border bg-card";

export function InstanceCard({ inst, index, isFirst, isLast, onUpdate, onToggleConditional, onToggleVisibility, onToggleCollapsed, onMoveUp, onMoveDown, onDuplicate, onRemove, buffsSection, buffsSetCount = 0 }: IInstanceCardProps): React.ReactElement {
    const t: CardT = useT("tools");
    const { op, config, color, visible, collapsed } = inst;
    const [buffsOpen, setBuffsOpen] = React.useState(false);
    const detail = useOperatorDetail(op);
    const rarity = detail.raw ? rarityToNumber(detail.raw.rarity) : null;

    const promotion = config.promotion ?? Math.max(0, detail.phaseCount - 1);
    const maxLevel = detail.maxLevelForPromotion(promotion);
    const level = config.level ?? maxLevel;
    const maxRank = maxSkillRankForElite(promotion);

    const skillSummary = detail.skillName(config.skillIndex);
    const moduleSummary = config.moduleIndex > 0 ? detail.moduleName(config.moduleIndex) : t("calc.detail.noModule");

    const optionalModules = op.availableModules.filter((m) => m > 0);

    return (
        <Card>
            <CardHeader className="flex grid-rows-1 flex-row items-center gap-2.5 px-4 py-3">
                <span aria-hidden="true" className="inline-flex size-8 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted font-semibold text-[10px]" style={{ boxShadow: `inset 0 0 0 2px ${color}` }}>
                    <OperatorAvatar charId={op.id} name={op.name} />
                </span>
                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-1.5">
                        <span className="truncate font-semibold text-[13px] leading-tight">{op.name}</span>
                        {/* Rarity in its ink once the operator has loaded: 4.85 to 4.98 on the light card, 7.58+ dark. */}
                        {rarity !== null && (
                            <span className="shrink-0 font-mono text-[10.5px] leading-tight" style={{ color: rarityInk(rarity) }}>
                                {RARITY_LABELS[rarity]}
                            </span>
                        )}
                        <span className="shrink-0 rounded bg-muted px-1 py-0.5 font-mono text-[9.5px] text-muted-foreground">#{index + 1}</span>
                    </div>
                    <div className="truncate text-[11px] text-muted-foreground">{t("calc.instance.buildSummary", { skill: skillSummary, rank: skillRankLabel(config.skillRank, t), module: moduleSummary })}</div>
                </div>
                <div className="flex shrink-0 items-center gap-0.5">
                    <IconButton label={visible ? t("calc.instance.hideCurve") : t("calc.instance.showCurve")} onClick={onToggleVisibility}>
                        {visible ? <Eye /> : <EyeOff />}
                    </IconButton>
                    {!isFirst && (
                        <span className="hidden sm:inline-flex">
                            <IconButton label={t("calc.instance.moveUp")} onClick={onMoveUp}>
                                <ArrowUp />
                            </IconButton>
                        </span>
                    )}
                    {!isLast && (
                        <span className="hidden sm:inline-flex">
                            <IconButton label={t("calc.instance.moveDown")} onClick={onMoveDown}>
                                <ArrowDown />
                            </IconButton>
                        </span>
                    )}
                    <IconButton label={t("calc.instance.duplicate")} onClick={onDuplicate}>
                        <Copy />
                    </IconButton>
                    <IconButton label={collapsed ? t("calc.instance.expand") : t("calc.instance.collapse")} expanded={!collapsed} onClick={onToggleCollapsed}>
                        {collapsed ? <ChevronDown /> : <ChevronUp />}
                    </IconButton>
                    <IconButton label={t("calc.instance.remove")} onClick={onRemove}>
                        <X />
                    </IconButton>
                </div>
            </CardHeader>
            {!collapsed && (
                <CardPanel className="space-y-3.5 px-4 pt-0 pb-3">
                    {detail.failed && <LoadFailed layout="inline" message={t("calc.instance.detailFailed")} onRetry={detail.retry} />}
                    <FieldRow label={t("calc.instance.promotion")}>
                        <ToggleGroup
                            aria-label={t("calc.instance.promotion")}
                            className="flex-wrap gap-1.5"
                            value={[String(promotion)]}
                            onValueChange={(next) => {
                                if (next[0] === undefined) return;
                                const p = Number(next[0]);
                                const newMax = detail.maxLevelForPromotion(p);
                                const wasAtMax = config.level === undefined || config.level >= maxLevel;
                                // Clamp both level and skill rank to what the new promotion allows.
                                onUpdate({ promotion: p, level: wasAtMax ? undefined : Math.min(level, newMax), skillRank: clampRankToElite(config.skillRank, p) });
                            }}
                        >
                            {Array.from({ length: detail.phaseCount }, (_, i) => i).map((p) => (
                                <IconChip key={`prom-${p}`} value={String(p)} label={`E${p}`} iconURL={eliteIcon(p)} />
                            ))}
                        </ToggleGroup>
                    </FieldRow>

                    <div>
                        <div className="mb-1.5 flex items-baseline justify-between">
                            <Label className="text-subtle-foreground text-xs leading-none">{t("calc.instance.level")}</Label>
                            <span className="font-mono text-[10.5px] text-foreground tabular-nums">
                                {level}
                                <span className="text-muted-foreground"> / {maxLevel}</span>
                            </span>
                        </div>
                        <Slider min={1} max={maxLevel} step={1} value={[level]} onValueChange={(v) => onUpdate({ level: Array.isArray(v) ? (v[0] ?? 1) : v })} />
                    </div>

                    <FieldRow label={t("calc.instance.potential")}>
                        <ToggleGroup aria-label={t("calc.instance.potential")} className="flex-wrap gap-1" value={[String(config.potential)]} onValueChange={(next) => next[0] !== undefined && onUpdate({ potential: Number(next[0]) })}>
                            {[1, 2, 3, 4, 5, 6].map((p) => (
                                <Tooltip key={`pot-${p}`}>
                                    <TooltipTrigger render={<ToggleGroupItem value={String(p)} aria-label={t("calc.instance.potentialAria", { rank: p })} className={cn(CHIP, "pointer-coarse:size-11 size-8 min-w-0 p-0 sm:size-8")} />}>
                                        <img alt="" aria-hidden="true" className="icon-theme-aware size-5 object-contain" decoding="async" loading="lazy" src={potentialIcon(p - 1)} />
                                    </TooltipTrigger>
                                    <TooltipPopup className="max-w-64">{detail.potentialLabel(p)}</TooltipPopup>
                                </Tooltip>
                            ))}
                        </ToggleGroup>
                    </FieldRow>

                    {op.availableSkills.length > 0 && (
                        <FieldRow label={t("calc.instance.skill")}>
                            <ToggleGroup aria-label={t("calc.instance.skill")} className="grid w-full grid-cols-3 gap-1.5" value={[String(config.skillIndex)]} onValueChange={(next) => next[0] !== undefined && onUpdate({ skillIndex: Number(next[0]) })}>
                                {op.availableSkills.map((s) => {
                                    const skill = detail.skillAt(s);
                                    const skillName = skill?.static?.levels?.[0]?.name ?? t("calc.instance.skillFallback", { index: s });
                                    return (
                                        <Tooltip key={`skill-${s}`}>
                                            <TooltipTrigger render={<ToggleGroupItem value={String(s)} aria-label={t("calc.instance.useSkill", { skill: skillName })} className={cn(CHIP, "h-auto pointer-coarse:min-h-11 min-w-0 flex-col gap-1 whitespace-normal px-1 py-1.5 sm:h-auto")} />}>
                                                {skill ? (
                                                    <img alt="" aria-hidden="true" className="size-7 shrink-0 object-contain" decoding="async" loading="lazy" src={skillIconURL(skill)} />
                                                ) : (
                                                    <span className="flex size-7 shrink-0 items-center justify-center rounded bg-muted font-mono text-[10px] text-muted-foreground">S{s}</span>
                                                )}
                                                <span className="line-clamp-1 break-all text-center font-medium text-[10.5px] leading-tight">{skillName}</span>
                                            </TooltipTrigger>
                                            <TooltipPopup className="max-w-56">{t("calc.detail.designatorWithName", { label: `S${s}`, name: skillName })}</TooltipPopup>
                                        </Tooltip>
                                    );
                                })}
                            </ToggleGroup>
                        </FieldRow>
                    )}

                    <FieldRow label={t("calc.instance.skillLevel")}>
                        <ToggleGroup aria-label={t("calc.instance.skillLevel")} className="flex-wrap items-center gap-1" value={[String(config.skillRank)]} onValueChange={(next) => next[0] !== undefined && onUpdate({ skillRank: Number(next[0]) })}>
                            {SKILL_LEVELS.map((lvl) => (
                                <RankChip key={`lvl-${lvl}`} rank={lvl} lockReason={rankLockReason(lvl, promotion, t)} t={t} />
                            ))}
                            <span aria-hidden="true" className="mx-0.5 h-6 w-px shrink-0 bg-border" />
                            {MASTERY_RANKS.map((m) => (
                                <RankChip key={`m-${m}`} rank={m} lockReason={rankLockReason(m, promotion, t)} t={t} />
                            ))}
                        </ToggleGroup>
                        {maxRank < 10 && <p className="mt-1 text-[10px] text-subtle-foreground leading-snug">{maxRank < 7 ? t("calc.instance.rankHint.e1") : t("calc.instance.rankHint.e2")}</p>}
                    </FieldRow>

                    {optionalModules.length > 0 && (
                        <FieldRow label={t("calc.instance.module")}>
                            <ToggleGroup aria-label={t("calc.instance.module")} className="flex-wrap gap-1.5" value={[String(config.moduleIndex)]} onValueChange={(next) => next[0] !== undefined && onUpdate({ moduleIndex: Number(next[0]) })}>
                                <ToggleGroupItem value="0" className={cn(CHIP, "h-auto pointer-coarse:min-h-11 min-w-16 px-2 py-1.5 font-medium text-[11px] sm:h-auto sm:text-[11px]")}>
                                    {t("calc.detail.noModule")}
                                </ToggleGroupItem>
                                {optionalModules.map((m) => {
                                    const mod = detail.moduleAt(m);
                                    const typeLabel = mod?.typeName1 && mod?.typeName2 ? `${mod.typeName1}-${mod.typeName2}` : (mod?.uniEquipName ?? t("calc.detail.moduleFallback", { index: m }));
                                    return (
                                        <Tooltip key={`mod-${m}`}>
                                            <TooltipTrigger render={<ToggleGroupItem value={String(m)} className={cn(CHIP, "h-auto pointer-coarse:min-h-11 gap-1.5 px-1.5 py-1 sm:h-auto")} />}>
                                                {mod ? (
                                                    <img alt="" aria-hidden="true" className="size-7 shrink-0 object-contain" decoding="async" loading="lazy" src={moduleIconURL(mod)} />
                                                ) : (
                                                    <span className="flex size-7 shrink-0 items-center justify-center rounded bg-muted font-mono text-[10px] text-muted-foreground">M{m}</span>
                                                )}
                                                <span className="font-medium font-mono text-[10.5px]">{typeLabel}</span>
                                            </TooltipTrigger>
                                            <TooltipPopup className="max-w-56">{mod?.uniEquipName ?? t("calc.instance.moduleFallbackTooltip", { index: m })}</TooltipPopup>
                                        </Tooltip>
                                    );
                                })}
                            </ToggleGroup>
                        </FieldRow>
                    )}

                    {config.moduleIndex > 0 && (
                        <FieldRow label={t("calc.instance.moduleLevel")}>
                            <ToggleGroup aria-label={t("calc.instance.moduleLevel")} className="gap-1" value={[String(config.moduleLevel)]} onValueChange={(next) => next[0] !== undefined && onUpdate({ moduleLevel: Number(next[0]) })}>
                                {[1, 2, 3].map((l) => (
                                    <ToggleGroupItem key={`ml-${l}`} value={String(l)} className={cn(CHIP, "h-8 pointer-coarse:h-11 min-w-10 pointer-coarse:min-w-11 px-2 font-medium font-mono text-[11px] sm:h-8 sm:min-w-10 sm:text-[11px]")}>
                                        L{l}
                                    </ToggleGroupItem>
                                ))}
                            </ToggleGroup>
                        </FieldRow>
                    )}

                    <div>
                        <div className="mb-1.5 flex items-baseline justify-between">
                            <Label className="text-subtle-foreground text-xs leading-none">{t("calc.instance.trust")}</Label>
                            <span className="font-mono text-[10.5px] text-foreground tabular-nums">{config.trust}%</span>
                        </div>
                        <Slider min={0} max={100} step={5} value={[config.trust]} onValueChange={(v) => onUpdate({ trust: Array.isArray(v) ? (v[0] ?? 0) : v })} />
                    </div>

                    {op.conditionals.length > 0 && (
                        <div>
                            <h4 className="mb-2 font-semibold text-foreground text-sm leading-none">{t("calc.instance.conditionals")}</h4>
                            <div className="space-y-1">
                                {op.conditionals.map((cond) => {
                                    const key = conditionalKey(cond);
                                    if (!key) return null;
                                    // The label describes the switch ON; an inverted conditional's label holds while the flag is off.
                                    const checked = conditionalSwitchOn(cond, config.conditionals[key]);
                                    return (
                                        <div key={`${cond.conditionalType}-${cond.name}`} className="flex items-center justify-between gap-2 rounded-md border border-border bg-muted/30 px-2 py-1.5">
                                            <div className="min-w-0 flex-1">
                                                <div className="truncate font-medium text-[12px] leading-tight" title={cond.name}>
                                                    {conditionalLabel(t, op.id, cond)}
                                                </div>
                                                <div className="mt-0.5 flex flex-wrap items-center gap-1">
                                                    <ScopeTag>{cond.conditionalType}</ScopeTag>
                                                    {cond.skills.map((s) => (
                                                        <ScopeTag key={`s-${s}`}>S{s}</ScopeTag>
                                                    ))}
                                                    {cond.modules.map((m) => (
                                                        <ScopeTag key={`m-${m}`}>Mod{m}</ScopeTag>
                                                    ))}
                                                </div>
                                            </div>
                                            <Switch checked={checked} onCheckedChange={(v) => onToggleConditional(key, conditionalFlagFor(cond, v))} />
                                        </div>
                                    );
                                })}
                            </div>
                        </div>
                    )}

                    {buffsSection && (
                        <Collapsible open={buffsOpen} onOpenChange={setBuffsOpen}>
                            <CollapsibleTrigger className="flex pointer-coarse:min-h-11 w-full items-center justify-between rounded py-1 text-left font-medium text-[11px] text-muted-foreground outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring">
                                <span className="flex items-center gap-1.5">
                                    {t("calc.instance.externalBuffs")}
                                    {buffsSetCount > 0 && (
                                        <Badge variant="outline" size="sm" className="font-mono">
                                            {t("calc.instance.externalBuffs.count", { count: buffsSetCount })}
                                        </Badge>
                                    )}
                                </span>
                                {buffsOpen ? <ChevronUp aria-hidden="true" className="size-3.5" /> : <ChevronDown aria-hidden="true" className="size-3.5" />}
                            </CollapsibleTrigger>
                            <CollapsibleContent>
                                <div className="pt-1.5">{buffsSection}</div>
                            </CollapsibleContent>
                        </Collapsible>
                    )}
                </CardPanel>
            )}
        </Card>
    );
}

interface IRankChipProps {
    rank: number;
    lockReason: string | null;
    t: CardT;
}

/** One skill rank in the rank group; a rank the promotion has not reached is disabled and says why. */
function RankChip({ rank, lockReason, t }: IRankChipProps): React.ReactElement {
    const locked = lockReason !== null;
    const mastery = isMasteryRank(rank);
    const tip = locked ? lockReason : skillRankLabel(rank, t);
    return (
        <Tooltip>
            <TooltipTrigger render={<ToggleGroupItem value={String(rank)} disabled={locked} aria-label={skillRankLabel(rank, t)} className={cn(CHIP, "h-8 pointer-coarse:h-11 min-w-8 pointer-coarse:min-w-11 px-1.5 disabled:opacity-40 sm:h-8 sm:min-w-8")} />}>
                {mastery ? <img alt="" aria-hidden="true" className="icon-theme-aware size-5 object-contain" decoding="async" loading="lazy" src={specializedIcon(rank - 7)} /> : <span className="font-medium font-mono text-[12px]">{rank}</span>}
            </TooltipTrigger>
            <TooltipPopup>{tip}</TooltipPopup>
        </Tooltip>
    );
}

function FieldRow({ label, children }: { label: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div>
            <Label className="mb-1.5 block text-subtle-foreground text-xs leading-none">{label}</Label>
            {children}
        </div>
    );
}

interface IIconChipProps {
    value: string;
    label: string;
    iconURL: string;
}

function IconChip({ value, label, iconURL }: IIconChipProps): React.ReactElement {
    return (
        <ToggleGroupItem value={value} aria-label={label} className={cn(CHIP, "pointer-coarse:size-11 size-9 min-w-0 p-0 sm:size-9")}>
            <img alt="" aria-hidden="true" className="icon-theme-aware size-6 object-contain" decoding="async" loading="lazy" src={iconURL} />
        </ToggleGroupItem>
    );
}

function IconButton({ label, expanded, onClick, children }: { label: string; expanded?: boolean; onClick: () => void; children: React.ReactNode }): React.ReactElement {
    return (
        <Tooltip>
            <TooltipTrigger
                render={(p) => (
                    <Button {...p} aria-label={label} aria-expanded={expanded} size="icon-xs" variant="ghost" onClick={onClick}>
                        {children}
                    </Button>
                )}
            />
            <TooltipPopup>{label}</TooltipPopup>
        </Tooltip>
    );
}

function ScopeTag({ children }: { children: React.ReactNode }): React.ReactElement {
    return <span className="rounded bg-background px-1 py-0.5 font-mono text-[9.5px] text-muted-foreground">{children}</span>;
}
