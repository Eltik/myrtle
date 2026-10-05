import { useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import { Layers, Trash, X } from "lucide-react";
import * as React from "react";

import { eliteIcon, specializedIcon } from "#/components/operators/detail/impl/assets";
import { Button, type IButtonProps } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { useErrorMessage } from "#/components/ui/error-message";
import { Input } from "#/components/ui/input";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Switch } from "#/components/ui/switch";
import { useAuth } from "#/hooks/use-auth";
import { operatorQueryOptions } from "#/lib/api/operators";
import { deletePlanPresetFn, type IOperatorPlanResponse, type IPresetTarget, PLAN_PRESETS_QUERY_KEY, planPresetsQueryOptions, plansQueryOptions, upsertPlanFn, upsertPlanPresetFn } from "#/lib/api/planner";
import { type IRosterEntry, userRosterQueryOptions } from "#/lib/api/user";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./BulkPlanDialog.messages";
import { clampPresetForOperator, DEFAULT_PRESET_TARGET, type IBulkPlanTarget, normalizePresetTarget, PRESET_LEVEL_CAPS, raiseToRoster, withPresetChange } from "./bulkTargets";
import type { messages as dialogMessages } from "./OperatorPlannerDialog.messages";
import { type IOperatorOption, OperatorMultiSelector, useOperatorOptions } from "./OperatorSelector";
import { PlanGroupsField } from "./PlanGroupsField";
import { stepStateClass } from "./PlanTargetSections";
import { ELITE_PHASES, MAX_SKILL_LEVEL, MODULE_STAGES, PLANS_QUERY_PREFIX, UNPLANNABLE_OPERATOR_ID } from "./planTargets";

type BulkT = TypedT<typeof messages & typeof dialogMessages>;

const SKILL_LEVELS = Array.from({ length: MAX_SKILL_LEVEL }, (_, i) => i + 1);
const MASTERY_RANKS = [0, 1, 2, 3] as const;
const SKILL_SLOTS = [0, 1, 2] as const;

/** Where one picked operator stands: what it will be saved as, or why it is skipped. */
type RowStatus = { kind: "loading" } | { kind: "error"; message: string } | { kind: "unplannable" } | { kind: "planned" } | { kind: "reached" } | { kind: "ready"; target: IBulkPlanTarget; existing: IOperatorPlanResponse | undefined };

interface IPreviewRow {
    option: IOperatorOption;
    status: RowStatus;
}

interface IBulkPlanButtonProps {
    size?: IButtonProps["size"];
    variant?: IButtonProps["variant"];
    className?: string;
}

/** The "Bulk add" button with the dialog it opens; it owns the open state so the planner page only mounts this. */
export function BulkPlanButton({ size = "sm", variant = "outline", className }: IBulkPlanButtonProps): React.ReactElement {
    const t: BulkT = useT("tools");
    const [open, setOpen] = React.useState<boolean>(false);
    return (
        <>
            <Button size={size} variant={variant} className={className} onClick={() => setOpen(true)}>
                <Layers className="mr-1.5 size-4" />
                {t("planner.bulk.open")}
            </Button>
            <BulkPlanDialog open={open} onOpenChange={setOpen} />
        </>
    );
}

interface IBulkPlanDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

/**
 * Plans many operators with one target. Each operator gets the target clamped
 * to its rarity and kit (`clampPresetForOperator`), raised to its roster
 * progress (`raiseToRoster`), and the preview shows exactly what will be
 * saved before anything is. The plans, groups and roster come from the same
 * queries the single plan dialog reads, so they are already cached.
 */
export function BulkPlanDialog({ open, onOpenChange }: IBulkPlanDialogProps): React.ReactElement {
    const t: BulkT = useT("tools");
    const errorMessage = useErrorMessage();
    const server = useGamedataServer();
    const { user } = useAuth();
    const queryClient = useQueryClient();

    const [selected, setSelected] = React.useState<IOperatorOption[]>([]);
    const [searchQuery, setSearchQuery] = React.useState<string>("");
    const [target, setTarget] = React.useState<IPresetTarget>(DEFAULT_PRESET_TARGET);
    const [groups, setGroups] = React.useState<string[]>([]);
    const [overwrite, setOverwrite] = React.useState<boolean>(false);
    const [isSaving, setIsSaving] = React.useState<boolean>(false);
    const [failures, setFailures] = React.useState<Record<string, string>>({});

    React.useEffect(() => {
        if (open) return;
        setSelected([]);
        setSearchQuery("");
        setTarget(DEFAULT_PRESET_TARGET);
        setGroups([]);
        setOverwrite(false);
        setIsSaving(false);
        setFailures({});
    }, [open]);

    const { options, isLoading: isOptionsLoading } = useOperatorOptions(server, searchQuery, null);
    const { data: roster, isSuccess: isRosterLoaded } = useQuery({
        ...userRosterQueryOptions(user?.uid ?? ""),
        enabled: !!user?.uid,
    });
    const { data: plannerData, isSuccess: isPlansLoaded } = useQuery({
        ...plansQueryOptions(),
        enabled: !!user?.uid,
    });
    const details = useQueries({
        queries: selected.map((op) => ({ ...operatorQueryOptions(op.id, server), enabled: open })),
    });

    // Planned operators only matter when their plans are being replaced.
    const plannedIds = React.useMemo(() => new Set((plannerData?.plans ?? []).map((p) => p.operator_id)), [plannerData?.plans]);
    const pickableOptions = React.useMemo(() => (overwrite ? options : options.filter((op) => !plannedIds.has(op.id))), [options, overwrite, plannedIds]);

    const handleOverwriteChange = (checked: boolean) => {
        setOverwrite(checked);
        if (!checked) setSelected((prev) => prev.filter((op) => !plannedIds.has(op.id)));
    };

    const groupNames = React.useMemo(() => (plannerData?.groups ?? []).map((g) => g.name), [plannerData?.groups]);

    const rows = React.useMemo<IPreviewRow[]>(() => {
        const plansById = new Map((plannerData?.plans ?? []).map((p) => [p.operator_id, p]));
        const rosterById = new Map<string, IRosterEntry>((roster ?? []).map((entry) => [entry.operator_id, entry]));
        const hasRoster = !user?.uid || isRosterLoaded;
        return selected.map((option, idx): IPreviewRow => {
            if (option.id === UNPLANNABLE_OPERATOR_ID) return { option, status: { kind: "unplannable" } };
            const detail = details[idx];
            if (detail?.isError) return { option, status: { kind: "error", message: errorMessage(detail.error) } };
            const operator = detail?.data;
            if (!operator || !hasRoster || !isPlansLoaded) return { option, status: { kind: "loading" } };
            const existing = plansById.get(option.id);
            if (existing && !overwrite) return { option, status: { kind: "planned" } };
            const merged = raiseToRoster(clampPresetForOperator(target, operator), rosterById.get(option.id));
            if (merged.isReached) return { option, status: { kind: "reached" } };
            return { option, status: { kind: "ready", target: merged.target, existing } };
        });
    }, [selected, details, plannerData?.plans, roster, user?.uid, isRosterLoaded, isPlansLoaded, overwrite, target, errorMessage]);

    const readyCount = rows.filter((r) => r.status.kind === "ready").length;
    const isAnyLoading = rows.some((r) => r.status.kind === "loading");
    const failureCount = rows.filter((r) => failures[r.option.id]).length;

    const removeOperator = (id: string) => setSelected((prev) => prev.filter((op) => op.id !== id));

    const handleSave = async () => {
        const ready = rows.flatMap((r) => (r.status.kind === "ready" ? [{ option: r.option, status: r.status }] : []));
        if (ready.length === 0) return;
        setIsSaving(true);
        setFailures({});
        const results = await Promise.allSettled(
            ready.map(({ option, status }) =>
                upsertPlanFn({
                    data: {
                        operatorId: option.id,
                        ...status.target,
                        // A replaced plan gains the chosen groups, and switching the profile off never hides one already shown.
                        displayOnProfile: target.display_on_profile || (status.existing?.display_on_profile ?? false),
                        groups: [...new Set([...(status.existing?.groups ?? []), ...groups])],
                    },
                }),
            ),
        );

        const nextFailures: Record<string, string> = {};
        const savedIds = new Set<string>();
        results.forEach((result, idx) => {
            const id = ready[idx]?.option.id;
            if (!id) return;
            if (result.status === "fulfilled") savedIds.add(id);
            else nextFailures[id] = errorMessage(result.reason);
        });

        if (savedIds.size > 0) queryClient.invalidateQueries({ queryKey: PLANS_QUERY_PREFIX });
        setIsSaving(false);
        if (Object.keys(nextFailures).length === 0) {
            onOpenChange(false);
            return;
        }
        setFailures(nextFailures);
        setSelected((prev) => prev.filter((op) => !savedIds.has(op.id)));
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogPopup bottomStickOnMobile={false} className="flex h-[min(840px,calc(100vh-4rem))] w-full max-w-[min(1152px,calc(100vw-2rem))] flex-col overflow-hidden p-0">
                <DialogHeader>
                    <DialogTitle>{t("planner.bulk.title")}</DialogTitle>
                    <DialogDescription>{t("planner.bulk.desc")}</DialogDescription>
                </DialogHeader>

                <DialogPanel className="min-h-0 flex-1">
                    <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
                        <div className="space-y-6">
                            <PresetRow target={target} onLoad={setTarget} />
                            <TargetForm target={target} onChange={setTarget} />
                            <PlanGroupsField groupNames={groupNames} selectedGroups={groups} onSelectedGroupsChange={setGroups} />
                            <SwitchRow id="bulk-display-on-profile" label={t("planner.dialog.displayOnProfile")} description={t("planner.bulk.displayOnProfile.desc")} checked={target.display_on_profile} onCheckedChange={(checked) => setTarget((prev) => ({ ...prev, display_on_profile: checked }))} />
                            <SwitchRow id="bulk-overwrite" label={t("planner.bulk.overwrite")} description={t("planner.bulk.overwrite.desc")} checked={overwrite} onCheckedChange={handleOverwriteChange} />
                        </div>

                        <div className="flex min-w-0 flex-col gap-3">
                            <div className="flex items-center justify-between gap-2">
                                <label className="flex items-center gap-2 font-medium text-[13px] text-muted-foreground leading-none" htmlFor="bulk-operator-selector">
                                    {t("planner.bulk.operators")}
                                    {selected.length > 0 && <span className="font-normal text-xs">{t("planner.bulk.selectedCount", { count: selected.length })}</span>}
                                </label>
                                {selected.length > 0 && (
                                    <Button variant="ghost" size="xs" onClick={() => setSelected([])}>
                                        {t("planner.bulk.clear")}
                                    </Button>
                                )}
                            </div>
                            <OperatorMultiSelector options={pickableOptions} selectedOptions={selected} isLoading={isOptionsLoading} onSelectedChange={setSelected} searchQuery={searchQuery} onSearchQueryChange={setSearchQuery} />

                            {failureCount > 0 && (
                                <div role="alert" className="rounded-lg border border-destructive/30 bg-destructive/8 px-3 py-2 text-destructive-foreground text-xs">
                                    {t("planner.bulk.failed", { count: failureCount })}
                                </div>
                            )}

                            <span className="mt-2 font-semibold text-muted-foreground text-xs uppercase tracking-wider">{t("planner.bulk.preview")}</span>
                            {rows.length === 0 ? (
                                <p className="text-muted-foreground text-sm italic">{t("planner.bulk.previewEmpty")}</p>
                            ) : (
                                <ul className="divide-y divide-border rounded-xl border border-border bg-card">
                                    {rows.map((row) => (
                                        <PreviewRow key={row.option.id} row={row} failure={failures[row.option.id]} onRemove={() => removeOperator(row.option.id)} />
                                    ))}
                                </ul>
                            )}
                        </div>
                    </div>
                </DialogPanel>

                <DialogFooter className="pb-8 sm:pb-8">
                    <DialogClose render={<Button variant="outline" className="w-full sm:w-auto" />}>{t("planner.dialog.cancel")}</DialogClose>
                    <Button onClick={handleSave} disabled={isSaving || isAnyLoading || readyCount === 0} className="w-full sm:w-auto">
                        {isSaving ? t("planner.dialog.saving") : t("planner.bulk.save", { count: readyCount })}
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}

interface IPresetRowProps {
    target: IPresetTarget;
    onLoad: (target: IPresetTarget) => void;
}

/** Load, save and delete named presets. Each action is saved straight away, independent of the dialog's Save. */
function PresetRow({ target, onLoad }: IPresetRowProps): React.ReactElement {
    const t: BulkT = useT("tools");
    const errorMessage = useErrorMessage();
    const queryClient = useQueryClient();
    const { data: presets = [] } = useQuery(planPresetsQueryOptions());
    const [chosen, setChosen] = React.useState<string | null>(null);
    const [name, setName] = React.useState<string>("");
    const [isBusy, setIsBusy] = React.useState<boolean>(false);
    const [error, setError] = React.useState<string | null>(null);

    const sorted = React.useMemo(() => [...presets].sort((a, b) => a.name.localeCompare(b.name)), [presets]);
    const trimmed = name.trim();
    const isReplace = sorted.some((p) => p.name === trimmed);

    const run = async (action: () => Promise<unknown>) => {
        setIsBusy(true);
        setError(null);
        try {
            await action();
            await queryClient.invalidateQueries({ queryKey: PLAN_PRESETS_QUERY_KEY });
            return true;
        } catch (err) {
            setError(errorMessage(err));
            return false;
        } finally {
            setIsBusy(false);
        }
    };

    const handleChoose = (value: string | null) => {
        setChosen(value);
        const preset = sorted.find((p) => p.name === value);
        if (!preset) return;
        onLoad(normalizePresetTarget(preset.target));
        setName(preset.name);
    };

    const handleSave = async () => {
        if (!trimmed) return;
        if (await run(() => upsertPlanPresetFn({ data: { name: trimmed, target } }))) setChosen(trimmed);
    };

    const handleDelete = async () => {
        if (!chosen) return;
        const deleted = chosen;
        if (await run(() => deletePlanPresetFn({ data: deleted }))) {
            setChosen(null);
            if (name.trim() === deleted) setName("");
        }
    };

    return (
        <div className="space-y-3 rounded-xl border border-border bg-card/40 p-4">
            <span className="font-semibold text-muted-foreground text-xs uppercase tracking-wider">{t("planner.bulk.preset")}</span>
            <div className="flex items-center gap-2">
                <Select value={chosen} onValueChange={(v: string | null) => handleChoose(v)}>
                    <SelectTrigger className="min-w-0 flex-1" aria-label={t("planner.bulk.preset")}>
                        <SelectValue placeholder={t("planner.bulk.presetPlaceholder")}>{(value: string | null) => value ?? t("planner.bulk.presetPlaceholder")}</SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                        {sorted.length === 0 ? (
                            <p className="px-2 py-1.5 text-muted-foreground text-xs">{t("planner.bulk.presetNone")}</p>
                        ) : (
                            sorted.map((preset) => (
                                <SelectItem key={preset.id} value={preset.name}>
                                    {preset.name}
                                </SelectItem>
                            ))
                        )}
                    </SelectContent>
                </Select>
                <Button variant="outline" size="icon" onClick={handleDelete} disabled={!chosen || isBusy} aria-label={t("planner.bulk.presetDelete")} title={t("planner.bulk.presetDelete")}>
                    <Trash className="size-4" />
                </Button>
            </div>
            <div className="flex items-center gap-2">
                <Input
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder={t("planner.bulk.presetName")}
                    maxLength={100}
                    className="min-w-0 flex-1"
                    onKeyDown={(e) => {
                        if (e.key === "Enter") {
                            e.preventDefault();
                            handleSave();
                        }
                    }}
                />
                <Button variant="outline" onClick={handleSave} disabled={!trimmed || isBusy}>
                    {isReplace ? t("planner.bulk.presetReplace") : t("planner.bulk.presetSave")}
                </Button>
            </div>
            {error && (
                <p role="alert" className="text-destructive-foreground text-xs">
                    {error}
                </p>
            )}
        </div>
    );
}

interface ITargetFormProps {
    target: IPresetTarget;
    onChange: (target: IPresetTarget) => void;
}

interface ISwitchRowProps {
    id: string;
    label: string;
    description: string;
    checked: boolean;
    onCheckedChange: (checked: boolean) => void;
}

function SwitchRow({ id, label, description, checked, onCheckedChange }: ISwitchRowProps): React.ReactElement {
    return (
        <div className="flex items-center justify-between gap-4 rounded-xl border border-border bg-card/40 p-4">
            <div className="space-y-0.5">
                <label htmlFor={id} className="cursor-pointer font-semibold text-foreground text-sm">
                    {label}
                </label>
                <p className="text-muted-foreground text-xs">{description}</p>
            </div>
            <Switch id={id} checked={checked} onCheckedChange={onCheckedChange} />
        </div>
    );
}

/** The rarity-free target: promotion, level, shared skill level, a mastery per skill slot and one module stage. */
function TargetForm({ target, onChange }: ITargetFormProps): React.ReactElement {
    const t: BulkT = useT("tools");
    const cap = PRESET_LEVEL_CAPS[target.elite] ?? 90;

    const changeLevel = (raw: number) => onChange(withPresetChange(target, { field: "level", value: Number.isNaN(raw) ? 1 : raw }));

    return (
        <div className="space-y-5 rounded-xl border border-border bg-card/40 p-4">
            <span className="font-semibold text-muted-foreground text-xs uppercase tracking-wider">{t("planner.bulk.target")}</span>

            <div className="grid grid-cols-1 gap-5 sm:grid-cols-[auto_1fr]">
                <FieldBlock label={t("planner.dialog.promotion")}>
                    <div className="flex items-center gap-2">
                        {ELITE_PHASES.map((e) => (
                            <button
                                key={e}
                                type="button"
                                onClick={() => onChange(withPresetChange(target, { field: "elite", value: e }))}
                                className={cn("flex size-12 cursor-pointer items-center justify-center rounded-lg border transition-all", stepStateClass(target.elite === e))}
                                title={t("planner.dialog.elite", { elite: e })}
                            >
                                <img src={eliteIcon(e)} alt={t("planner.dialog.elite", { elite: e })} className="icon-theme-aware size-7 object-contain" />
                            </button>
                        ))}
                    </div>
                </FieldBlock>

                <FieldBlock label={t("planner.bulk.level")}>
                    <Input type="number" min={1} max={cap} value={target.level ?? cap} onChange={(e) => changeLevel(Number.parseInt(e.target.value, 10))} className="w-20 text-center font-mono" aria-label={t("planner.bulk.level")} />
                    <p className="text-muted-foreground text-xs">{t("planner.bulk.levelCapNote", { max: cap })}</p>
                </FieldBlock>
            </div>

            <FieldBlock label={t("planner.bulk.skillLevel")}>
                <div className="flex flex-wrap gap-1.5">
                    {SKILL_LEVELS.map((value) => (
                        <StepButton key={value} isActive={target.skill_level === value} title={t("planner.dialog.skillLevel", { level: value })} onSelect={() => onChange(withPresetChange(target, { field: "skill_level", value }))}>
                            <span className="font-semibold text-[13px]">{value}</span>
                        </StepButton>
                    ))}
                </div>
            </FieldBlock>

            <FieldBlock label={t("planner.bulk.masteries")}>
                <div className="space-y-1.5">
                    {SKILL_SLOTS.map((slot) => (
                        <div key={slot} className="flex items-center gap-3">
                            <span className="w-6 font-semibold text-muted-foreground text-xs">{t("planner.bulk.skillSlot", { index: slot + 1 })}</span>
                            <div className="flex gap-1.5">
                                {MASTERY_RANKS.map((rank) => (
                                    <StepButton key={rank} isActive={target.masteries[slot] === rank} title={rank === 0 ? t("planner.dialog.notPlanned") : t("planner.dialog.mastery", { mastery: rank })} onSelect={() => onChange(withPresetChange(target, { field: "mastery", index: slot, value: rank }))}>
                                        {rank === 0 ? <span className="font-semibold text-[13px]">-</span> : <img src={specializedIcon(rank)} alt={t("planner.dialog.mastery", { mastery: rank })} className="icon-theme-aware size-6 object-contain" />}
                                    </StepButton>
                                ))}
                            </div>
                        </div>
                    ))}
                </div>
            </FieldBlock>

            <FieldBlock label={t("planner.bulk.moduleStage")}>
                <div className="flex gap-1.5">
                    {MODULE_STAGES.map((stage) => (
                        <StepButton key={stage} isActive={target.module_stage === stage} title={stage === 0 ? t("planner.dialog.notPlanned") : t("planner.dialog.stage", { stage })} onSelect={() => onChange(withPresetChange(target, { field: "module_stage", value: stage }))}>
                            <span className="font-semibold text-[13px]">{stage === 0 ? "-" : stage}</span>
                        </StepButton>
                    ))}
                </div>
            </FieldBlock>

            <p className="text-muted-foreground text-xs">{t("planner.bulk.targetNote")}</p>
        </div>
    );
}

function FieldBlock({ label, children }: { label: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div className="flex flex-col gap-2">
            <span className="font-medium text-[13px] text-muted-foreground leading-none">{label}</span>
            {children}
        </div>
    );
}

interface IStepButtonProps {
    isActive: boolean;
    title: string;
    onSelect: () => void;
    children: React.ReactNode;
}

function StepButton({ isActive, title, onSelect, children }: IStepButtonProps): React.ReactElement {
    return (
        <button type="button" onClick={onSelect} aria-pressed={isActive} className={cn("flex size-9 shrink-0 cursor-pointer items-center justify-center rounded-md border transition-all", stepStateClass(isActive))} title={title}>
            {children}
        </button>
    );
}

interface IPreviewRowProps {
    row: IPreviewRow;
    /** The save error for this operator, if its last save failed. */
    failure: string | undefined;
    onRemove: () => void;
}

/** One picked operator: its clamped target, or why it is skipped. */
function PreviewRow({ row, failure, onRemove }: IPreviewRowProps): React.ReactElement {
    const t: BulkT = useT("tools");
    const { option, status } = row;

    const statusText = (): React.ReactNode => {
        switch (status.kind) {
            case "loading":
                return <span className="text-muted-foreground">{t("planner.bulk.status.loading")}</span>;
            case "error":
                return <span className="text-destructive-foreground">{status.message}</span>;
            case "unplannable":
                return <span className="text-muted-foreground">{t("planner.bulk.status.unplannable")}</span>;
            case "planned":
                return <span className="text-muted-foreground">{t("planner.bulk.status.planned")}</span>;
            case "reached":
                return <span className="text-muted-foreground">{t("planner.bulk.status.reached")}</span>;
            case "ready":
                return <TargetSummary target={status.target} replaces={!!status.existing} />;
        }
    };

    return (
        <li className={cn("flex items-center gap-3 px-3 py-2", status.kind !== "ready" && "opacity-70")}>
            <span aria-hidden="true" className="relative flex size-9 shrink-0 items-center justify-center overflow-hidden rounded-lg border border-border bg-muted/50">
                <OperatorAvatar charId={option.id} name={option.displayName} className="block h-full w-full object-cover" server={option.isUpcoming ? "cn" : undefined} />
            </span>
            <div className="min-w-0 flex-1">
                <p className="truncate font-medium text-foreground text-sm">{option.displayName}</p>
                <p className="text-xs">{statusText()}</p>
                {failure && <p className="text-destructive-foreground text-xs">{failure}</p>}
            </div>
            <Button variant="ghost" size="icon-xs" onClick={onRemove} aria-label={t("planner.bulk.remove", { operator: option.displayName })}>
                <X />
            </Button>
        </li>
    );
}

function TargetSummary({ target, replaces }: { target: IBulkPlanTarget; replaces: boolean }): React.ReactElement {
    const t: BulkT = useT("tools");
    const masteries = target.targetSkills.map((s) => s.mastery_level);
    const stages = target.targetModules.map((m) => m.module_stage);
    const parts = [t("planner.bulk.summary.promotion", { elite: target.targetElite, level: target.targetLevel }), t("planner.bulk.summary.skill", { level: target.targetSkillLevel })];
    if (masteries.some((m) => m > 0)) parts.push(t("planner.bulk.summary.masteries", { masteries: masteries.join("/") }));
    if (stages.some((s) => s > 0)) parts.push(t("planner.bulk.summary.module", { stage: stages.join("/") }));

    return (
        <span className="flex flex-wrap items-center gap-x-2 font-mono text-foreground/80">
            <span>{parts.join(" · ")}</span>
            {replaces && <span className="font-sans text-amber-600 dark:text-amber-400">{t("planner.bulk.status.replaces")}</span>}
        </span>
    );
}
