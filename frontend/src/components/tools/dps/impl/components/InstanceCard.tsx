import { ArrowDown, ArrowUp, ChevronDown, ChevronUp, Copy, Eye, EyeOff, X } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { Card, CardHeader, CardPanel } from "#/components/ui/card";
import { Collapsible, CollapsibleContent } from "#/components/ui/collapsible";
import { Label } from "#/components/ui/label";
import { NumberField, NumberFieldDecrement, NumberFieldGroup, NumberFieldIncrement, NumberFieldInput } from "#/components/ui/number-field";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Slider } from "#/components/ui/slider";
import { Switch } from "#/components/ui/switch";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import type { IDpsCalculateBuffs, IDpsCalculateConditionals, IDpsConditionalInfo } from "#/lib/api/dps";
import { cn } from "#/lib/utils";
import { eliteIcon, moduleIconURL, potentialIcon, skillIconURL, specializedIcon } from "../icons";
import type { IDpsInstance, IDpsInstanceConfig } from "../types";
import { useOperatorDetail } from "../useOperatorDetail";

interface IInstanceCardProps {
    inst: IDpsInstance;
    index: number;
    isFirst: boolean;
    isLast: boolean;
    onUpdate: (patch: Partial<IDpsInstanceConfig>) => void;
    onUpdateBuffs: (patch: Partial<IDpsCalculateBuffs>) => void;
    onToggleConditional: (key: keyof IDpsCalculateConditionals, value: boolean) => void;
    onToggleVisibility: () => void;
    onToggleCollapsed: () => void;
    onMoveUp: () => void;
    onMoveDown: () => void;
    onDuplicate: () => void;
    onRemove: () => void;
}

export function InstanceCard({ inst, index, isFirst, isLast, onUpdate, onUpdateBuffs, onToggleConditional, onToggleVisibility, onToggleCollapsed, onMoveUp, onMoveDown, onDuplicate, onRemove }: IInstanceCardProps): React.ReactElement {
    const { op, config, color, visible, collapsed } = inst;
    const [buffsOpen, setBuffsOpen] = React.useState(false);
    const detail = useOperatorDetail(op.id);

    const promotion = config.promotion ?? Math.max(0, detail.phaseCount - 1);
    const maxLevel = detail.maxLevelForPromotion(promotion);
    const level = config.level ?? maxLevel;

    const skillSummary = detail.skillName(config.skillIndex);
    const moduleSummary = config.moduleIndex > 0 ? detail.moduleName(config.moduleIndex) : "No module";

    const optionalModules = op.availableModules.filter((m) => m > 0);

    return (
        <Card>
            <CardHeader className="flex flex-row items-center gap-2.5 px-4 py-3 grid-rows-1">
                <span aria-hidden="true" className="inline-flex size-8 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted text-[10px] font-semibold" style={{ boxShadow: `inset 0 0 0 2px ${color}` }}>
                    <OperatorAvatar charId={op.id} name={op.name} />
                </span>
                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-1.5">
                        <span className="truncate font-semibold text-[13px] leading-tight">{op.name}</span>
                        <span className="shrink-0 rounded bg-muted px-1 py-0.5 font-mono text-[9.5px] text-muted-foreground">#{index + 1}</span>
                    </div>
                    <div className="truncate text-[11px] text-muted-foreground">
                        {skillSummary} · {moduleSummary}
                    </div>
                </div>
                <div className="flex shrink-0 items-center gap-0.5">
                    <IconButton label={visible ? "Hide curve" : "Show curve"} onClick={onToggleVisibility}>
                        {visible ? <Eye /> : <EyeOff />}
                    </IconButton>
                    {/* Reorder controls take space, so hide on the smallest screens — duplicate/remove still let users prune */}
                    {!isFirst && (
                        <span className="hidden sm:inline-flex">
                            <IconButton label="Move up" onClick={onMoveUp}>
                                <ArrowUp />
                            </IconButton>
                        </span>
                    )}
                    {!isLast && (
                        <span className="hidden sm:inline-flex">
                            <IconButton label="Move down" onClick={onMoveDown}>
                                <ArrowDown />
                            </IconButton>
                        </span>
                    )}
                    <IconButton label="Duplicate" onClick={onDuplicate}>
                        <Copy />
                    </IconButton>
                    <IconButton label={collapsed ? "Expand" : "Collapse"} onClick={onToggleCollapsed}>
                        {collapsed ? <ChevronDown /> : <ChevronUp />}
                    </IconButton>
                    <IconButton label="Remove" onClick={onRemove}>
                        <X />
                    </IconButton>
                </div>
            </CardHeader>
            {!collapsed && (
                <CardPanel className="space-y-3.5 px-4 pt-0 pb-3">
                    <FieldRow label="Promotion">
                        <div className="flex flex-wrap gap-1.5">
                            {Array.from({ length: detail.phaseCount }, (_, i) => i).map((p) => (
                                <IconChip
                                    key={`prom-${p}`}
                                    selected={promotion === p}
                                    label={`E${p}`}
                                    iconURL={eliteIcon(p)}
                                    onClick={() => {
                                        const newMax = detail.maxLevelForPromotion(p);
                                        // If `level` was at the previous max (or undefined → "max" sentinel), keep
                                        // it as undefined so the new promotion's max applies. Otherwise clamp to
                                        // the new max so we never exceed the cap.
                                        const wasAtMax = config.level === undefined || config.level >= maxLevel;
                                        onUpdate({ promotion: p, level: wasAtMax ? undefined : Math.min(level, newMax) });
                                    }}
                                />
                            ))}
                        </div>
                    </FieldRow>

                    <div>
                        <div className="mb-1.5 flex items-baseline justify-between">
                            <Label className="font-medium text-[11px] leading-none text-muted-foreground">Level</Label>
                            <span className="font-mono text-[10.5px] text-foreground tabular-nums">
                                {level}
                                <span className="text-muted-foreground"> / {maxLevel}</span>
                            </span>
                        </div>
                        <Slider min={1} max={maxLevel} step={1} value={[level]} onValueChange={(v) => onUpdate({ level: Array.isArray(v) ? (v[0] ?? 1) : v })} />
                    </div>

                    <FieldRow label="Potential">
                        <div className="flex flex-wrap gap-1">
                            {[1, 2, 3, 4, 5, 6].map((p) => (
                                <Tooltip key={`pot-${p}`}>
                                    <TooltipTrigger
                                        render={(triggerProps) => (
                                            <button
                                                {...triggerProps}
                                                type="button"
                                                onClick={() => onUpdate({ potential: p })}
                                                aria-label={`Potential ${p}`}
                                                className={cn(
                                                    "flex size-8 shrink-0 cursor-pointer items-center justify-center rounded-md border bg-card transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background",
                                                    config.potential === p ? "border-primary bg-primary/10 ring-1 ring-primary/30" : "border-border hover:border-primary/50",
                                                )}
                                            >
                                                <img alt="" aria-hidden="true" className="size-5 object-contain" decoding="async" loading="lazy" src={potentialIcon(p - 1)} />
                                            </button>
                                        )}
                                    />
                                    <TooltipPopup className="max-w-64">{detail.potentialLabel(p)}</TooltipPopup>
                                </Tooltip>
                            ))}
                        </div>
                    </FieldRow>

                    {op.availableSkills.length > 0 && (
                        <FieldRow label="Skill">
                            <div className="grid grid-cols-3 gap-1.5">
                                {op.availableSkills.map((s) => {
                                    const skill = detail.skillAt(s);
                                    const skillName = skill?.static?.levels?.[0]?.name ?? `Skill ${s}`;
                                    return (
                                        <Tooltip key={`skill-${s}`}>
                                            <TooltipTrigger
                                                render={(triggerProps) => (
                                                    <button
                                                        {...triggerProps}
                                                        type="button"
                                                        onClick={() => onUpdate({ skillIndex: s })}
                                                        aria-label={`Use ${skillName}`}
                                                        className={cn(
                                                            "flex min-w-0 cursor-pointer flex-col items-center gap-1 rounded-md border bg-card px-1 py-1.5 transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background",
                                                            config.skillIndex === s ? "border-primary bg-primary/10 ring-1 ring-primary/30" : "border-border hover:border-primary/50",
                                                        )}
                                                    >
                                                        {skill ? (
                                                            <img alt="" aria-hidden="true" className="size-7 shrink-0 object-contain" decoding="async" loading="lazy" src={skillIconURL(skill)} />
                                                        ) : (
                                                            <span className="flex size-7 shrink-0 items-center justify-center rounded bg-muted font-mono text-[10px] text-muted-foreground">S{s}</span>
                                                        )}
                                                        <span className="line-clamp-1 break-all text-center text-[10.5px] font-medium leading-tight">{skillName}</span>
                                                    </button>
                                                )}
                                            />
                                            <TooltipPopup className="max-w-56">
                                                S{s} · {skillName}
                                            </TooltipPopup>
                                        </Tooltip>
                                    );
                                })}
                            </div>
                        </FieldRow>
                    )}

                    <FieldRow label="Mastery">
                        <div className="flex flex-wrap gap-1">
                            {[0, 1, 2, 3].map((m) => (
                                <Tooltip key={`mastery-${m}`}>
                                    <TooltipTrigger
                                        render={(triggerProps) => (
                                            <button
                                                {...triggerProps}
                                                type="button"
                                                onClick={() => onUpdate({ masteryLevel: m })}
                                                aria-label={m === 0 ? "No mastery (Lv 7)" : `Mastery ${m}`}
                                                className={cn(
                                                    "flex h-8 cursor-pointer items-center justify-center gap-1 rounded-md border bg-card px-2 transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background",
                                                    config.masteryLevel === m ? "border-primary bg-primary/10 ring-1 ring-primary/30" : "border-border hover:border-primary/50",
                                                )}
                                            >
                                                {m === 0 ? <span className="font-mono text-[11px] font-medium">L7</span> : <img alt="" aria-hidden="true" className="size-5 object-contain" decoding="async" loading="lazy" src={specializedIcon(m)} />}
                                            </button>
                                        )}
                                    />
                                    <TooltipPopup>{m === 0 ? "Skill Lv 7 (no mastery)" : `Mastery ${m}`}</TooltipPopup>
                                </Tooltip>
                            ))}
                        </div>
                    </FieldRow>

                    {optionalModules.length > 0 && (
                        <FieldRow label="Module">
                            <div className="flex flex-wrap gap-1.5">
                                <button
                                    type="button"
                                    onClick={() => onUpdate({ moduleIndex: 0 })}
                                    className={cn(
                                        "flex min-w-16 cursor-pointer items-center justify-center rounded-md border bg-card px-2 py-1.5 text-[11px] font-medium transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background",
                                        config.moduleIndex === 0 ? "border-primary bg-primary/10 ring-1 ring-primary/30" : "border-border hover:border-primary/50",
                                    )}
                                >
                                    No module
                                </button>
                                {optionalModules.map((m) => {
                                    const mod = detail.moduleAt(m);
                                    const typeLabel = mod?.typeName1 && mod?.typeName2 ? `${mod.typeName1}-${mod.typeName2}` : (mod?.uniEquipName ?? `Mod ${m}`);
                                    return (
                                        <Tooltip key={`mod-${m}`}>
                                            <TooltipTrigger
                                                render={(triggerProps) => (
                                                    <button
                                                        {...triggerProps}
                                                        type="button"
                                                        onClick={() => onUpdate({ moduleIndex: m })}
                                                        className={cn(
                                                            "flex cursor-pointer items-center gap-1.5 rounded-md border bg-card px-1.5 py-1 transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background",
                                                            config.moduleIndex === m ? "border-primary bg-primary/10 ring-1 ring-primary/30" : "border-border hover:border-primary/50",
                                                        )}
                                                    >
                                                        {mod ? (
                                                            <img alt="" aria-hidden="true" className="size-7 shrink-0 object-contain" decoding="async" loading="lazy" src={moduleIconURL(mod)} />
                                                        ) : (
                                                            <span className="flex size-7 shrink-0 items-center justify-center rounded bg-muted font-mono text-[10px] text-muted-foreground">M{m}</span>
                                                        )}
                                                        <span className="font-mono text-[10.5px] font-medium">{typeLabel}</span>
                                                    </button>
                                                )}
                                            />
                                            <TooltipPopup className="max-w-56">{mod?.uniEquipName ?? `Module ${m}`}</TooltipPopup>
                                        </Tooltip>
                                    );
                                })}
                            </div>
                        </FieldRow>
                    )}

                    {config.moduleIndex > 0 && (
                        <FieldRow label="Module level">
                            <div className="flex gap-1">
                                {[1, 2, 3].map((l) => (
                                    <button
                                        key={`ml-${l}`}
                                        type="button"
                                        onClick={() => onUpdate({ moduleLevel: l })}
                                        className={cn(
                                            "flex h-8 min-w-10 cursor-pointer items-center justify-center rounded-md border bg-card px-2 font-mono text-[11px] font-medium transition-colors",
                                            config.moduleLevel === l ? "border-primary bg-primary/10 ring-1 ring-primary/30" : "border-border hover:border-primary/50",
                                        )}
                                    >
                                        L{l}
                                    </button>
                                ))}
                            </div>
                        </FieldRow>
                    )}

                    <div>
                        <div className="mb-1.5 flex items-baseline justify-between">
                            <Label className="font-medium text-[11px] leading-none text-muted-foreground">Trust</Label>
                            <span className="font-mono text-[10.5px] text-foreground tabular-nums">{config.trust}%</span>
                        </div>
                        <Slider min={0} max={100} step={5} value={[config.trust]} onValueChange={(v) => onUpdate({ trust: Array.isArray(v) ? (v[0] ?? 0) : v })} />
                    </div>

                    {op.conditionals.length > 0 && (
                        <div>
                            <Label className="mb-1.5 block font-medium text-[11px] leading-none text-muted-foreground">Conditionals</Label>
                            <div className="space-y-1">
                                {op.conditionals.map((cond) => {
                                    const key = mapConditionalKey(cond);
                                    if (!key) return null;
                                    const checked = Boolean(config.conditionals[key]);
                                    return (
                                        <div key={`${cond.conditionalType}-${cond.name}`} className="flex items-center justify-between gap-2 rounded-md border border-border bg-muted/30 px-2 py-1.5">
                                            <div className="min-w-0 flex-1">
                                                <div className="truncate font-medium text-[12px] leading-tight">{cond.name}</div>
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
                                            <Switch checked={checked} onCheckedChange={(v) => onToggleConditional(key, v)} />
                                        </div>
                                    );
                                })}
                            </div>
                        </div>
                    )}

                    <Collapsible open={buffsOpen} onOpenChange={setBuffsOpen}>
                        <button type="button" className="flex w-full cursor-pointer items-center justify-between rounded py-1 text-left font-medium text-[11px] text-muted-foreground hover:text-foreground" onClick={() => setBuffsOpen((v) => !v)}>
                            External buffs
                            {buffsOpen ? <ChevronUp className="size-3.5" /> : <ChevronDown className="size-3.5" />}
                        </button>
                        <CollapsibleContent>
                            <div className="grid grid-cols-2 gap-2 pt-1.5">
                                <NumField label="ATK %" value={Math.round((config.buffs.atk ?? 0) * 100)} min={0} max={500} step={5} onChange={(v) => onUpdateBuffs({ atk: v / 100 })} />
                                <NumField label="Flat ATK" value={config.buffs.flatAtk ?? 0} min={0} max={2000} step={10} onChange={(v) => onUpdateBuffs({ flatAtk: v })} />
                                <NumField label="ASPD" value={config.buffs.aspd ?? 0} min={0} max={200} step={5} onChange={(v) => onUpdateBuffs({ aspd: v })} />
                                <NumField label="Fragile %" value={Math.round((config.buffs.fragile ?? 0) * 100)} min={0} max={300} step={5} onChange={(v) => onUpdateBuffs({ fragile: v / 100 })} />
                            </div>
                        </CollapsibleContent>
                    </Collapsible>
                </CardPanel>
            )}
        </Card>
    );
}

function FieldRow({ label, children }: { label: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div>
            <Label className="mb-1.5 block font-medium text-[11px] leading-none text-muted-foreground">{label}</Label>
            {children}
        </div>
    );
}

interface IconChipProps {
    selected: boolean;
    label: string;
    iconURL: string;
    onClick: () => void;
}

function IconChip({ selected, label, iconURL, onClick }: IconChipProps): React.ReactElement {
    return (
        <button
            type="button"
            onClick={onClick}
            aria-label={label}
            aria-pressed={selected}
            className={cn(
                "flex h-9 w-9 shrink-0 cursor-pointer items-center justify-center rounded-md border bg-card outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background",
                selected ? "border-primary bg-primary/10 ring-1 ring-primary/30" : "border-border hover:border-primary/50",
            )}
        >
            <img alt="" aria-hidden="true" className="size-6 object-contain icon-theme-aware" decoding="async" loading="lazy" src={iconURL} />
        </button>
    );
}

function IconButton({ label, onClick, children }: { label: string; onClick: () => void; children: React.ReactNode }): React.ReactElement {
    return (
        <Tooltip>
            <TooltipTrigger
                render={(p) => (
                    <Button {...p} aria-label={label} size="icon-xs" variant="ghost" onClick={onClick}>
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

function mapConditionalKey(cond: IDpsConditionalInfo): keyof IDpsCalculateConditionals | null {
    switch (cond.conditionalType) {
        case "trait":
            return "traitDamage";
        case "talent":
            return "talentDamage";
        case "talent2":
            return "talent2Damage";
        case "skill":
            return "skillDamage";
        case "module":
            return "moduleDamage";
        default:
            return null;
    }
}

interface NumFieldProps {
    label: string;
    value: number;
    min?: number;
    max?: number;
    step?: number;
    onChange: (value: number) => void;
}

function NumField({ label, value, min, max, step = 1, onChange }: NumFieldProps): React.ReactElement {
    return (
        <NumberField defaultValue={value} min={min} max={max} step={step} onValueChange={(v) => onChange(v ?? 0)} size="sm" className="gap-1">
            <Label className="block font-medium text-[11px] leading-none text-muted-foreground">{label}</Label>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
    );
}
