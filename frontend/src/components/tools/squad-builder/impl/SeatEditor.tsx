import { useQuery } from "@tanstack/react-query";
import { ChevronDown, FlaskConical, Trash2, Undo2, X } from "lucide-react";
import * as React from "react";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { eliteIcon, moduleIconURL, potentialIcon, skillIconURL } from "#/components/tools/shared/icons";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Checkbox } from "#/components/ui/checkbox";
import { Label } from "#/components/ui/label";
import { LoadFailed } from "#/components/ui/load-failed";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { Slider } from "#/components/ui/slider";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { operatorQueryOptions } from "#/lib/api/operators";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, getAvatarById, rarityStarColor } from "#/lib/utils";
import type { SquadBuild } from "#/types/generated/SquadBuild";
import type { SquadMember } from "#/types/generated/SquadMember";
import type { IOperatorIndexEntry } from "#/types/operators";
import { DPS_SQUADMATE_LIMIT } from "../handoff";
import { advancedModules, clampBuild, type IResolvedSeat, type IRosterBuild, levelCap, maxMastery, maxPhase, maxSkillLevel, moduleRankCount, moduleTypeLabel, reached, SUPPORT_SEAT, skillUnlock, topRank } from "../model";
import type { messages } from "./SeatEditor.messages";

type EditorT = TypedT<typeof messages>;

export interface ISquadmateOption {
    seat: number;
    charId: string;
    name: string;
    /** Gives this seat a buff the DPS calculator can take. */
    gives: boolean;
}

export interface ISeatEditorProps {
    seat: number;
    resolved: IResolvedSeat;
    op: IOperatorIndexEntry;
    roster: IRosterBuild | undefined;
    readOnly: boolean;
    squadmates: readonly ISquadmateOption[];
    labOpen: boolean;
    onLabOpenChange: (open: boolean) => void;
    onOpenLab: (picked: number[]) => void;
    onChange: (member: SquadMember) => void;
    onRemove: () => void;
    onClose: () => void;
}

/** A toggle tile's border: ringed when pressed, a hover hint when not. */
/** Build pickers are kit toggles drawn as separate bordered tiles, not a joined segment. */
const CHIP = "border-border bg-background/40";

/** The "no module" value in the module group; module ids are never empty. */
const NO_MODULE = "";

/** The slider reports one thumb as a number or a one-entry list. */
function sliderValue(v: number | readonly number[]): number {
    return Array.isArray(v) ? (v[0] ?? 1) : (v as number);
}

/** "PHASE_2" -> 2. */
function phaseNumber(phase: string | null | undefined): number {
    const n = Number(String(phase ?? "").replace(/\D/g, ""));
    return Number.isFinite(n) ? n : 0;
}

export function SeatEditor({ seat, resolved, op, roster, readOnly, squadmates, labOpen, onLabOpenChange, onOpenLab, onChange, onRemove, onClose }: ISeatEditorProps): React.ReactElement {
    const t: EditorT = useT("tools");
    const server = useGamedataServer();
    const { data: detail, isError: detailFailed, refetch: refetchDetail } = useQuery(operatorQueryOptions(op.id, server));
    const { member, build, source } = resolved;
    const isSupport = seat === SUPPORT_SEAT;
    const modules = advancedModules(op);
    const skillCount = Math.max(op.skillCount, detail?.skills?.length ?? 0);
    // One slider over ranks then masteries: rank r up to the top rank, then top + m for mastery m, as far as this build has unlocked.
    const top = topRank(op);
    const reachedRank = maxSkillLevel(op, build.phase, build.level);
    const maxRank = reachedRank + maxMastery(op, member.skill_index, build.phase, build.level, reachedRank);
    const rank = build.mastery > 0 ? top + build.mastery : build.skill_level;
    const cap = levelCap(op, build.phase);
    const moduleRanks = moduleRankCount(op, member.module_id);

    const setBuild = (patch: Partial<SquadBuild>) => {
        const next = clampBuild(op, { ...build, ...patch }, member);
        onChange({ ...member, build: next });
    };
    const setModule = (moduleId: string | null) => {
        const next: SquadMember = { ...member, module_id: moduleId };
        if (member.build) next.build = clampBuild(op, { ...member.build, module_level: moduleId ? member.build.module_level || moduleRankCount(op, moduleId) : 0 }, next);
        onChange(next);
    };

    const isLocked = (i: number) => {
        const u = skillUnlock(op, i);
        return u !== null && !reached(u, build.phase, build.level);
    };
    const skillLocked = isLocked(member.skill_index) ? skillUnlock(op, member.skill_index) : null;
    const chosenModule = member.module_id ? detail?.modules?.find((m) => m.uniEquipId === member.module_id) : undefined;
    const moduleUnlock = chosenModule ? { phase: phaseNumber(chosenModule.unlockEvolvePhase), level: chosenModule.unlockLevel } : null;
    const moduleLocked = moduleUnlock !== null && !reached(moduleUnlock, build.phase, build.level);
    const rosterDiffers = source === "hand" && roster !== undefined && (roster.elite !== build.phase || roster.level !== build.level);

    const sourceLabel = { roster: t("squad.editor.source.roster"), hand: t("squad.editor.source.hand"), full: t("squad.editor.source.full"), support: t("squad.editor.source.support") }[source];
    const sourceVariant = source === "roster" ? "success" : source === "hand" ? "info" : "secondary";

    return (
        <section aria-label={op.name} className="rounded-2xl border border-border bg-card p-4">
            <div className="flex items-start gap-3">
                <span className="relative size-14 shrink-0 overflow-hidden rounded-lg border border-border bg-muted" style={{ boxShadow: `inset 0 -3px 0 ${rarityStarColor(op.rarity)}` }}>
                    <img alt="" src={getAvatarById(op.id)} className="size-full object-cover" />
                </span>
                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-1.5">
                        <ClassIcon profession={op.profession} size={16} className="size-4!" />
                        <h3 className="truncate font-semibold text-[16px] leading-tight">{op.name}</h3>
                    </div>
                    <div className="mt-1 flex flex-wrap items-center gap-1.5">
                        <span className="font-mono text-[11px] text-muted-foreground">{isSupport ? t("squad.editor.supportSeat") : t("squad.editor.seat", { seat: seat + 1 })}</span>
                        <Badge variant={sourceVariant}>{sourceLabel}</Badge>
                    </div>
                </div>
                <Button variant="ghost" size="icon-sm" aria-label={t("squad.editor.close")} onClick={onClose}>
                    <X />
                </Button>
            </div>

            {detailFailed && <LoadFailed layout="inline" className="mt-3" message={t("squad.editor.detailFailed")} onRetry={() => void refetchDetail()} />}
            {readOnly && <p className="mt-3 rounded-lg border border-border bg-muted/40 px-3 py-2 text-[12.5px] text-muted-foreground">{t("squad.editor.readOnly")}</p>}

            <fieldset disabled={readOnly} className="mt-4 grid gap-x-6 gap-y-4 disabled:opacity-80 md:grid-cols-2">
                <Field label={t("squad.editor.skill")}>
                    <ToggleGroup aria-label={t("squad.editor.skill")} className="flex-wrap gap-1.5" value={[String(member.skill_index)]} onValueChange={(next) => next[0] !== undefined && onChange({ ...member, skill_index: Number(next[0]) })}>
                        {[...Array(skillCount).keys()].map((i) => {
                            const skill = detail?.skills?.[i];
                            const name = skill?.static?.levels?.[0]?.name ?? t("squad.editor.skillFallback", { skill: i + 1 });
                            return (
                                <Tooltip key={`skill-${i}`}>
                                    <TooltipTrigger render={<ToggleGroupItem value={String(i)} aria-label={name} className={cn(CHIP, "size-12 overflow-hidden p-0 sm:size-12", isLocked(i) && "opacity-60")} />}>
                                        {skill ? <img alt="" src={skillIconURL(skill)} className="size-full object-contain" /> : <span className="font-mono text-[11px] text-muted-foreground">S{i + 1}</span>}
                                    </TooltipTrigger>
                                    <TooltipPopup>{name}</TooltipPopup>
                                </Tooltip>
                            );
                        })}
                    </ToggleGroup>
                    {skillLocked !== null && <Hint>{t("squad.editor.hint.skill", { skill: member.skill_index + 1, phase: skillLocked.phase })}</Hint>}
                </Field>

                <Field label={t("squad.editor.skillLevel")} value={rank > top ? t("squad.editor.rankMastery", { mastery: rank - top }) : t("squad.editor.rankLevel", { rank })}>
                    <Slider
                        min={1}
                        max={Math.max(1, maxRank)}
                        step={1}
                        value={[Math.min(rank, maxRank)]}
                        onValueChange={(v) => {
                            const r = sliderValue(v);
                            setBuild({ skill_level: Math.min(top, r), mastery: Math.max(0, r - top) });
                        }}
                    />
                </Field>

                {modules.length > 0 && (
                    <Field label={t("squad.editor.module")}>
                        <ToggleGroup aria-label={t("squad.editor.module")} className="flex-wrap gap-1.5" value={[member.module_id ?? NO_MODULE]} onValueChange={(next) => next[0] !== undefined && setModule(next[0] === NO_MODULE ? null : next[0])}>
                            <ToggleGroupItem value={NO_MODULE} className={cn(CHIP, "h-12 min-w-14 px-2 font-medium text-[12px] sm:h-12 sm:text-[12px]")}>
                                {t("squad.editor.noModule")}
                            </ToggleGroupItem>
                            {modules.map((m) => {
                                const full = detail?.modules?.find((d) => d.uniEquipId === m.uniEquipId);
                                const label = moduleTypeLabel(m);
                                return (
                                    <Tooltip key={m.uniEquipId}>
                                        <TooltipTrigger render={<ToggleGroupItem value={m.uniEquipId} aria-label={label} className={cn(CHIP, "h-12 gap-1.5 px-1.5 sm:h-12")} />}>
                                            {full ? <img alt="" src={moduleIconURL(full)} className="size-9 object-contain" /> : null}
                                            <span className="font-medium font-mono text-[11px]">{label}</span>
                                        </TooltipTrigger>
                                        <TooltipPopup>{full?.uniEquipName ?? label}</TooltipPopup>
                                    </Tooltip>
                                );
                            })}
                        </ToggleGroup>
                        {moduleLocked && moduleUnlock && <Hint>{t("squad.editor.hint.module", { phase: moduleUnlock.phase, level: moduleUnlock.level })}</Hint>}
                    </Field>
                )}

                {member.module_id !== null && (
                    <Field label={t("squad.editor.moduleRank")}>
                        <Segmented label={t("squad.editor.moduleRank")} options={Array.from({ length: moduleRanks }, (_, i) => i + 1)} value={build.module_level} onChange={(v) => setBuild({ module_level: v })} render={(v) => String(v)} />
                    </Field>
                )}

                <Field label={t("squad.editor.promotion")}>
                    <ToggleGroup
                        aria-label={t("squad.editor.promotion")}
                        className="gap-1.5"
                        value={[String(build.phase)]}
                        onValueChange={(next) => {
                            if (next[0] === undefined) return;
                            const p = Number(next[0]);
                            setBuild({ phase: p, level: build.phase === p ? build.level : Math.min(build.level, levelCap(op, p)) });
                        }}
                    >
                        {Array.from({ length: op.phaseMaxLevels.length }, (_, i) => i).map((p) => (
                            <ToggleGroupItem key={`phase-${p}`} value={String(p)} disabled={p > maxPhase(op)} className={cn(CHIP, "h-10 pointer-coarse:h-11 min-w-14 gap-1 px-2 font-medium font-mono text-[12px] disabled:opacity-40 sm:h-10 sm:min-w-14 sm:text-[12px]")}>
                                <img alt="" src={eliteIcon(p)} className="icon-theme-aware size-5 object-contain" />E{p}
                            </ToggleGroupItem>
                        ))}
                    </ToggleGroup>
                </Field>

                <Field label={t("squad.editor.level")} value={`${build.level} / ${cap}`}>
                    <Slider min={1} max={cap} step={1} value={[build.level]} onValueChange={(v) => setBuild({ level: sliderValue(v) })} />
                </Field>

                <Field label={t("squad.editor.potential")}>
                    <ToggleGroup aria-label={t("squad.editor.potential")} className="flex-wrap gap-1" value={[String(build.potential)]} onValueChange={(next) => next[0] !== undefined && setBuild({ potential: Number(next[0]) })}>
                        {[...Array(op.potentialRankCount + 1).keys()].map((i) => (
                            <ToggleGroupItem key={`pot-${i}`} value={String(i)} aria-label={t("squad.editor.potentialAria", { rank: i + 1 })} className={cn(CHIP, "pointer-coarse:size-11 size-9 min-w-0 p-0 sm:size-9")}>
                                <img alt="" src={potentialIcon(i)} className="icon-theme-aware size-5 object-contain" />
                            </ToggleGroupItem>
                        ))}
                    </ToggleGroup>
                    {rosterDiffers && roster && <Hint>{t("squad.editor.hint.roster", { phase: roster.elite, level: roster.level })}</Hint>}
                </Field>
            </fieldset>

            <div className="mt-4 flex flex-wrap items-center gap-2 border-border border-t pt-3">
                <LabSplitButton squadmates={squadmates} open={labOpen} onOpenChange={onLabOpenChange} onOpen={onOpenLab} t={t} />
                {!readOnly && member.build !== null && (isSupport || roster !== undefined) && (
                    <Button variant="outline" size="sm" onClick={() => onChange({ ...member, build: null })}>
                        <Undo2 />
                        {isSupport ? t("squad.editor.useFull") : t("squad.editor.useRoster")}
                    </Button>
                )}
                {!readOnly && (
                    <Button variant="destructive-outline" size="sm" className="ml-auto" onClick={onRemove}>
                        <Trash2 />
                        {t("squad.editor.remove")}
                    </Button>
                )}
            </div>
        </section>
    );
}

function LabSplitButton({ squadmates, open, onOpenChange, onOpen, t }: { squadmates: readonly ISquadmateOption[]; open: boolean; onOpenChange: (open: boolean) => void; onOpen: (picked: number[]) => void; t: EditorT }): React.ReactElement {
    const defaults = React.useMemo(
        () =>
            squadmates
                .filter((s) => s.gives)
                .slice(0, DPS_SQUADMATE_LIMIT)
                .map((s) => s.seat),
        [squadmates],
    );
    const [picked, setPicked] = React.useState<number[]>(defaults);
    // Re-seed when the popover opens, so it starts from the givers the analysis found.
    React.useEffect(() => {
        if (open) setPicked(defaults);
    }, [open, defaults]);

    const toggle = (seat: number, on: boolean) => setPicked((prev) => (on ? (prev.length >= DPS_SQUADMATE_LIMIT ? prev : [...prev, seat]) : prev.filter((s) => s !== seat)));

    return (
        <div className="flex items-center">
            <Button size="sm" className="rounded-e-none" onClick={() => onOpen(defaults)}>
                <FlaskConical />
                {t("squad.editor.lab")}
            </Button>
            <Popover open={open} onOpenChange={onOpenChange}>
                <PopoverTrigger
                    render={(p) => (
                        <Button {...p} size="sm" className="rounded-s-none border-s border-s-primary-foreground/20 px-1.5" aria-label={t("squad.editor.labPick")}>
                            <ChevronDown />
                        </Button>
                    )}
                />
                <PopoverPopup className="w-[min(320px,calc(100vw-2rem))]">
                    <h4 className="font-semibold text-[14px]">{t("squad.editor.labTitle")}</h4>
                    <p className="mt-1 text-[12px] text-muted-foreground leading-snug">{t("squad.editor.labDesc")}</p>
                    {squadmates.length === 0 ? (
                        <p className="mt-3 text-[12.5px] text-muted-foreground">{t("squad.editor.labNone")}</p>
                    ) : (
                        <ul className="mt-3 flex max-h-64 flex-col gap-1 overflow-y-auto">
                            {squadmates.map((s) => {
                                const checked = picked.includes(s.seat);
                                return (
                                    <li key={s.seat}>
                                        <Label className="flex cursor-pointer items-center gap-2 rounded-md px-1.5 py-1 text-[13px] hover:bg-accent">
                                            <Checkbox checked={checked} disabled={!checked && picked.length >= DPS_SQUADMATE_LIMIT} onCheckedChange={(v) => toggle(s.seat, v === true)} />
                                            <img alt="" src={getAvatarById(s.charId)} className="size-6 rounded object-cover" />
                                            <span className="min-w-0 flex-1 truncate">{s.name}</span>
                                            {s.gives && <span className="shrink-0 text-[10.5px] text-success-foreground">{t("squad.editor.labGives")}</span>}
                                        </Label>
                                    </li>
                                );
                            })}
                        </ul>
                    )}
                    <Button size="sm" className="mt-3 w-full" onClick={() => onOpen(picked)}>
                        {t("squad.editor.labGo", { count: picked.length })}
                    </Button>
                </PopoverPopup>
            </Popover>
        </div>
    );
}

function Field({ label, value, children }: { label: string; value?: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div className="min-w-0">
            <div className="mb-1.5 flex items-baseline justify-between gap-2">
                <span className="font-medium text-[11.5px] text-muted-foreground">{label}</span>
                {value && <span className="font-mono text-[11px] text-foreground tabular-nums">{value}</span>}
            </div>
            {children}
        </div>
    );
}

function Hint({ children }: { children: React.ReactNode }): React.ReactElement {
    return <p className="mt-1.5 text-[11.5px] text-warning-foreground leading-snug">{children}</p>;
}

function Segmented({ label, options, value, onChange, render }: { label: string; options: number[]; value: number; onChange: (v: number) => void; render: (v: number) => string }): React.ReactElement {
    return (
        <ToggleGroup aria-label={label} className="gap-1" value={[String(value)]} onValueChange={(next) => next[0] !== undefined && onChange(Number(next[0]))}>
            {options.map((o) => (
                <ToggleGroupItem key={o} value={String(o)} className={cn(CHIP, "h-9 pointer-coarse:h-11 min-w-10 pointer-coarse:min-w-11 px-2 font-medium font-mono text-[12px] sm:h-9 sm:min-w-10 sm:text-[12px]")}>
                    {render(o)}
                </ToggleGroupItem>
            ))}
        </ToggleGroup>
    );
}
