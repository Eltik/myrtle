import { useQuery } from "@tanstack/react-query";
import { ChevronDown } from "lucide-react";
import * as React from "react";

import { eliteIcon, moduleIconURL, skillIconURL, specializedIcon } from "#/components/operators/detail/impl/assets";
import { getSkillTypeLabel, getSpTypeLabel } from "#/components/operators/detail/impl/helpers";
import { Button } from "#/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Combobox, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup } from "#/components/ui/combobox";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Input } from "#/components/ui/input";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Slider } from "#/components/ui/slider";
import { Switch } from "#/components/ui/switch";
import { operatorsListQueryOptions } from "#/lib/api/operators";
import { professionLabel } from "#/lib/registry/operator-display";
import { scoreMatch } from "#/lib/search/fuzzy";
import { cn, formatSubProfession, rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";

function getMaxLevel(rarity: number, elite: number): number {
    if (rarity <= 2) return 30;
    if (rarity === 3) return elite === 0 ? 40 : 55;
    if (rarity === 4) return elite === 0 ? 45 : elite === 1 ? 60 : 70;
    if (rarity === 5) return elite === 0 ? 50 : elite === 1 ? 70 : 80;
    return elite === 0 ? 50 : elite === 1 ? 80 : 90;
}

interface IOperatorPlannerDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

/** Modal dialog for customizing operator targets. */
export function OperatorPlannerDialog({ open, onOpenChange }: IOperatorPlannerDialogProps): React.ReactElement {
    const [selectedOperator, setSelectedOperator] = React.useState<IOperatorListItem | null>(null);
    const [elite, setElite] = React.useState<number>(0);
    const [level, setLevel] = React.useState<number>(1);
    const [skillTargets, setSkillTargets] = React.useState<Record<number, number>>({});
    const [moduleTargets, setModuleTargets] = React.useState<Record<string, number>>({});
    const [displayOnProfile, setDisplayOnProfile] = React.useState<boolean>(false);
    const [skillsOpen, setSkillsOpen] = React.useState<boolean>(true);
    const [modulesSectionOpen, setModulesSectionOpen] = React.useState<boolean>(true);

    const { data: operators = [], isLoading } = useQuery(operatorsListQueryOptions());

    const sortedOperators = React.useMemo(() => {
        return [...operators].sort((a, b) => {
            const ra = rarityToNumber(a.rarity);
            const rb = rarityToNumber(b.rarity);
            if (rb !== ra) return rb - ra;
            return a.name.localeCompare(b.name);
        });
    }, [operators]);

    const opRarity = selectedOperator ? rarityToNumber(selectedOperator.rarity) : 6;
    const maxElite = opRarity <= 2 ? 0 : opRarity === 3 ? 1 : 2;
    const maxL = selectedOperator ? getMaxLevel(opRarity, elite) : 90;

    const filter = React.useCallback((op: IOperatorListItem, query: string) => {
        if (!query.trim()) return true;
        const target = {
            name: op.name,
            extra: `${op.appellation} ${professionLabel(op.profession)} ${op.subProfessionId} ${rarityToNumber(op.rarity)}★ ${(op.tagList ?? []).join(" ")} ${op.nationId}`,
        };
        return scoreMatch(query, target) > 0;
    }, []);

    React.useEffect(() => {
        if (!open) {
            setSelectedOperator(null);
            setElite(0);
            setLevel(1);
            setSkillTargets({});
            setModuleTargets({});
            setDisplayOnProfile(false);
            setSkillsOpen(true);
            setModulesSectionOpen(true);
        }
    }, [open]);

    React.useEffect(() => {
        if (selectedOperator) {
            const r = rarityToNumber(selectedOperator.rarity);
            const maxE = r <= 2 ? 0 : r === 3 ? 1 : 2;
            setElite((prev) => {
                const nextE = Math.min(prev, maxE);
                const maxLForElite = getMaxLevel(r, nextE);
                setLevel((l) => Math.min(l, maxLForElite));
                return nextE;
            });

            const initialSkills: Record<number, number> = {};
            selectedOperator.skills.forEach((_, idx) => {
                initialSkills[idx] = 1;
            });
            setSkillTargets(initialSkills);

            const initialModules: Record<string, number> = {};
            selectedOperator.modules.forEach((m) => {
                if (m.typeName1 !== "ORIGINAL") {
                    initialModules[m.uniEquipId] = 0;
                }
            });
            setModuleTargets(initialModules);
        }
    }, [selectedOperator]);

    const handleEliteChange = (newElite: number) => {
        setElite(newElite);
        const maxLForElite = getMaxLevel(opRarity, newElite);
        setLevel((prev) => Math.min(prev, maxLForElite));
    };

    const handleLevelChange = (val: number) => {
        const targetVal = Number.isNaN(val) ? 1 : val;
        const safeVal = Math.min(Math.max(1, targetVal), maxL);
        setLevel(safeVal);
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogPopup bottomStickOnMobile={false} className="flex h-[min(840px,calc(100vh-4rem))] w-full max-w-[min(1152px,calc(100vw-2rem))] flex-col overflow-hidden p-0">
                <DialogHeader>
                    <DialogTitle>Create new plan</DialogTitle>
                    <DialogDescription>Add a new operator target plan to your planner list.</DialogDescription>
                </DialogHeader>

                <DialogPanel className="min-h-0 flex-1">
                    <div className="space-y-6">
                        <div className="space-y-2">
                            <label className="block font-medium text-[13px] text-muted-foreground leading-none" htmlFor="operator-selector">
                                Select Operator
                            </label>
                            <Combobox<IOperatorListItem, false> items={sortedOperators} value={selectedOperator} onValueChange={setSelectedOperator} filter={filter} itemToStringLabel={(op) => op?.name ?? ""} itemToStringValue={(op) => op?.id ?? ""}>
                                <ComboboxInput id="operator-selector" placeholder={isLoading ? "Loading operators..." : "Search operators by name, class, or tag..."} />
                                <ComboboxPopup className="max-w-100">
                                    <ComboboxEmpty>No operators found.</ComboboxEmpty>
                                    <ComboboxList>
                                        {(op: IOperatorListItem) => {
                                            const rarity = rarityToNumber(op.rarity);
                                            return (
                                                <ComboboxItem key={op.id} value={op}>
                                                    <span className="flex w-full items-center gap-3">
                                                        <span aria-hidden="true" className="relative flex size-9 shrink-0 items-center justify-center overflow-hidden rounded-lg border border-border bg-muted/50">
                                                            <OperatorAvatar charId={op.id} name={op.name} className="block h-full w-full object-cover" />
                                                        </span>
                                                        <span className="flex-1 font-medium text-foreground text-sm">{op.name}</span>
                                                        <span className="font-normal text-muted-foreground text-xs">
                                                            {rarity}★ · {professionLabel(op.profession)}
                                                        </span>
                                                    </span>
                                                </ComboboxItem>
                                            );
                                        }}
                                    </ComboboxList>
                                </ComboboxPopup>
                            </Combobox>
                        </div>

                        {selectedOperator ? (
                            <div className="space-y-6">
                                <div className="rounded-xl border border-border bg-card p-4">
                                    <div className="flex items-center gap-4">
                                        <span aria-hidden="true" className="relative flex size-14 shrink-0 items-center justify-center overflow-hidden rounded-xl border border-border bg-muted/70">
                                            <OperatorAvatar charId={selectedOperator.id} name={selectedOperator.name} className="block h-full w-full object-cover" />
                                        </span>
                                        <div>
                                            <h3 className="font-bold text-foreground text-lg">{selectedOperator.name}</h3>
                                            <p className="text-muted-foreground text-xs">
                                                {rarityToNumber(selectedOperator.rarity)}★ · {formatSubProfession(selectedOperator.subProfessionId)}
                                            </p>
                                        </div>
                                    </div>
                                </div>

                                <div className="grid grid-cols-1 items-center gap-6 rounded-xl border border-border bg-card/40 p-4 sm:grid-cols-[auto_1fr]">
                                    <div className="flex flex-col gap-2">
                                        <span className="font-semibold text-muted-foreground text-xs uppercase tracking-wider">Promotion</span>
                                        <div className="mt-1 flex items-center gap-2">
                                            {[0, 1, 2].map((e) => {
                                                const isAvailable = e <= maxElite;
                                                const isActive = elite === e;
                                                if (!isAvailable) return null;
                                                return (
                                                    <button
                                                        key={e}
                                                        type="button"
                                                        onClick={() => handleEliteChange(e)}
                                                        className={cn(
                                                            "relative flex size-12 cursor-pointer items-center justify-center rounded-lg border transition-all",
                                                            isActive ? "border-primary bg-primary/10 opacity-100 ring-2 ring-primary/20" : "border-border bg-muted/40 opacity-40 hover:bg-muted/80 hover:opacity-75",
                                                        )}
                                                        title={`Elite ${e}`}
                                                    >
                                                        <img src={eliteIcon(e)} alt={`Elite ${e}`} className="icon-theme-aware size-7 object-contain" />
                                                    </button>
                                                );
                                            })}
                                        </div>
                                    </div>

                                    <div className="flex w-full flex-col gap-2">
                                        <span className="font-semibold text-muted-foreground text-xs uppercase tracking-wider">Level (Max {maxL})</span>
                                        <div className="mt-2 flex items-center gap-4">
                                            <Slider value={[level]} onValueChange={(vals) => handleLevelChange(Array.isArray(vals) ? (vals[0] ?? 1) : vals)} min={1} max={maxL} className="flex-1" />
                                            <Input type="number" min={1} max={maxL} value={level} onChange={(e) => handleLevelChange(parseInt(e.target.value, 10))} className="w-16 text-center font-mono" />
                                        </div>
                                    </div>
                                </div>

                                {selectedOperator.skills.length > 0 && (
                                    <Collapsible open={skillsOpen} onOpenChange={setSkillsOpen} className="space-y-3">
                                        <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                                            <span className="flex items-center gap-2">
                                                <span className="font-semibold text-foreground text-sm">Skills</span>
                                            </span>
                                            <ChevronDown className={cn("h-4 w-4 transition-transform", skillsOpen && "rotate-180")} />
                                        </CollapsibleTrigger>
                                        <CollapsibleContent>
                                            <div className="grid grid-cols-1 gap-4 pt-1">
                                                {selectedOperator.skills.map((skill, idx) => {
                                                    const currentTarget = skillTargets[idx] ?? 1;
                                                    const skillLevelInfo = skill.static?.levels?.[currentTarget - 1] ?? skill.static?.levels?.[0];
                                                    const name = skillLevelInfo?.name ?? `Skill ${idx + 1}`;
                                                    const recoveryType = skillLevelInfo?.spData?.spType ? getSpTypeLabel(skillLevelInfo.spData.spType) : "";
                                                    const triggerType = skillLevelInfo?.skillType ? getSkillTypeLabel(skillLevelInfo.skillType) : "";
                                                    const initialSp = skillLevelInfo?.spData?.initSp ?? 0;
                                                    const totalSp = skillLevelInfo?.spData?.spCost ?? 0;

                                                    return (
                                                        <div key={skill.skillId} className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-1.5 rounded-xl border border-border bg-card p-4 sm:grid-cols-[auto_1fr_auto]">
                                                            <img src={skillIconURL(skill)} alt={name} className="col-start-1 row-span-2 row-start-1 size-12 rounded-lg border border-border bg-muted/30 object-contain p-0.5" />

                                                            <div className="col-start-2 row-start-1 flex items-center gap-2">
                                                                <span className="font-bold text-base text-foreground leading-tight">{name}</span>
                                                            </div>

                                                            <div className="col-start-2 row-start-2 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-muted-foreground text-xs leading-none sm:hidden md:flex">
                                                                {recoveryType && (
                                                                    <>
                                                                        <span>{recoveryType}</span>
                                                                        <span className="text-muted-foreground/30" aria-hidden="true">
                                                                            •
                                                                        </span>
                                                                    </>
                                                                )}
                                                                {triggerType && <span>{triggerType}</span>}
                                                                {totalSp > 0 && (
                                                                    <>
                                                                        <span className="text-muted-foreground/30" aria-hidden="true">
                                                                            •
                                                                        </span>
                                                                        <span>
                                                                            SP: {initialSp}/{totalSp}
                                                                        </span>
                                                                    </>
                                                                )}
                                                            </div>

                                                            <div className="col-span-2 col-start-1 row-start-3 mt-2 grid grid-cols-10 gap-1 sm:col-span-1 sm:col-start-3 sm:row-span-2 sm:row-start-1 sm:mt-0 sm:flex sm:w-auto sm:justify-end sm:gap-1.5">
                                                                {Array.from({ length: 10 }, (_, i) => i + 1).map((val) => {
                                                                    const isMastery = val > 7;
                                                                    const isActive = currentTarget === val;

                                                                    return (
                                                                        <button
                                                                            key={val}
                                                                            type="button"
                                                                            onClick={() =>
                                                                                setSkillTargets((prev) => ({
                                                                                    ...prev,
                                                                                    [idx]: val,
                                                                                }))
                                                                            }
                                                                            className={cn(
                                                                                "flex aspect-square w-full shrink-0 cursor-pointer items-center justify-center rounded-md border transition-all sm:size-9",
                                                                                isActive ? "border-primary bg-primary/10 opacity-100 ring-2 ring-primary/20" : "border-border bg-muted/40 opacity-40 hover:bg-muted/80 hover:opacity-75",
                                                                            )}
                                                                            title={isMastery ? `Mastery ${val - 7}` : `Level ${val}`}
                                                                        >
                                                                            {isMastery ? <img src={specializedIcon(val - 7)} alt={`Mastery ${val - 7}`} className="size-5 object-contain sm:size-6" /> : <span className="font-semibold text-xs sm:text-[13px]">{val}</span>}
                                                                        </button>
                                                                    );
                                                                })}
                                                            </div>
                                                        </div>
                                                    );
                                                })}
                                            </div>
                                        </CollapsibleContent>
                                    </Collapsible>
                                )}

                                {selectedOperator.modules.filter((m) => m.typeName1 !== "ORIGINAL").length > 0 && (
                                    <Collapsible open={modulesSectionOpen} onOpenChange={setModulesSectionOpen} className="space-y-3">
                                        <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                                            <span className="flex items-center gap-2">
                                                <span className="font-semibold text-foreground text-sm">Modules</span>
                                            </span>
                                            <ChevronDown className={cn("h-4 w-4 transition-transform", modulesSectionOpen && "rotate-180")} />
                                        </CollapsibleTrigger>
                                        <CollapsibleContent>
                                            <div className="grid grid-cols-1 gap-4 pt-1">
                                                {selectedOperator.modules
                                                    .filter((m) => m.typeName1 !== "ORIGINAL")
                                                    .map((mod) => {
                                                        const currentTarget = moduleTargets[mod.uniEquipId] ?? 0;
                                                        const name = mod.uniEquipName;
                                                        const moduleTag = mod.typeName1 && mod.typeName2 ? `${mod.typeName1}-${mod.typeName2}` : (mod.typeName1 ?? "Module");

                                                        return (
                                                            <div key={mod.uniEquipId} className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-1.5 rounded-xl border border-border bg-card p-4 sm:grid-cols-[auto_1fr_auto]">
                                                                <img src={moduleIconURL(mod)} alt={name} className="col-start-1 row-span-2 row-start-1 size-12 rounded-lg border border-border bg-muted/30 object-contain p-0.5" />

                                                                <div className="col-start-2 row-start-1 flex items-center gap-2">
                                                                    <span className="font-bold text-base text-foreground leading-tight">{name}</span>
                                                                </div>

                                                                <div className="col-start-2 row-start-2 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-muted-foreground text-xs leading-none">
                                                                    <span>{moduleTag}</span>
                                                                </div>

                                                                <div className="col-span-2 col-start-1 row-start-3 mt-2 flex gap-1.5 sm:col-span-1 sm:col-start-3 sm:row-span-2 sm:row-start-1 sm:mt-0 sm:justify-end">
                                                                    {[0, 1, 2, 3].map((val) => {
                                                                        const isActive = currentTarget === val;
                                                                        return (
                                                                            <button
                                                                                key={val}
                                                                                type="button"
                                                                                onClick={() =>
                                                                                    setModuleTargets((prev) => ({
                                                                                        ...prev,
                                                                                        [mod.uniEquipId]: val,
                                                                                    }))
                                                                                }
                                                                                className={cn(
                                                                                    "flex size-9 shrink-0 cursor-pointer items-center justify-center rounded-md border transition-all",
                                                                                    isActive ? "border-primary bg-primary/10 opacity-100 ring-2 ring-primary/20" : "border-border bg-muted/40 opacity-40 hover:bg-muted/80 hover:opacity-75",
                                                                                )}
                                                                                title={val === 0 ? "Not planned" : `Stage ${val}`}
                                                                            >
                                                                                <span className="font-semibold text-xs sm:text-[13px]">{val === 0 ? "X" : val}</span>
                                                                            </button>
                                                                        );
                                                                    })}
                                                                </div>
                                                            </div>
                                                        );
                                                    })}
                                            </div>
                                        </CollapsibleContent>
                                    </Collapsible>
                                )}

                                <div className="flex items-center justify-between rounded-xl border border-border bg-card/40 p-4">
                                    <div className="space-y-0.5">
                                        <label htmlFor="display-on-profile" className="cursor-pointer font-semibold text-foreground text-sm">
                                            Display on profile
                                        </label>
                                        <p className="text-muted-foreground text-xs">Show this target plan on your public user profile.</p>
                                    </div>
                                    <Switch id="display-on-profile" checked={displayOnProfile} onCheckedChange={setDisplayOnProfile} />
                                </div>
                            </div>
                        ) : (
                            <p className="text-muted-foreground text-sm italic">Please select an operator to customize targets.</p>
                        )}
                    </div>
                </DialogPanel>

                <DialogFooter className="pb-8 sm:pb-8">
                    <DialogClose render={<Button variant="outline" className="w-full sm:w-auto" />}>Cancel</DialogClose>
                    <Button onClick={() => onOpenChange(false)} disabled={!selectedOperator} className="w-full sm:w-auto">
                        Save
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}
