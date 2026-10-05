import type * as React from "react";

import { eliteIcon, moduleIconURL, skillIconURL } from "#/components/operators/detail/impl/assets";
import { cn } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import type { IPlanProgress } from "./planProgress";

/**
 * The surface's own wording. The planner and the profile keep their copy in
 * different namespaces, and they label a few values differently (the profile
 * writes an unplanned module stage as "X", the planner as an em dash), so
 * every label comes in already translated.
 */
export interface IPlanProgressLabels {
    level: string;
    skills: string;
    modules: string;
    eliteAlt: (elite: number) => string;
    levelValue: (level: number) => string;
    skillFallback: (n: number) => string;
    /** A one-axis skill target: 1-7, then 8-10 for M1-M3. */
    skillTarget: (value: number) => string;
    moduleStage: (stage: number) => string;
}

interface IPlanProgressRowsProps {
    progress: IPlanProgress;
    /** The operator's asset server, for its skill and module icons. */
    server: IOperatorListItem["server"];
    labels: IPlanProgressLabels;
    className?: string;
}

/** Current ➔ target rows for a plan's promotion, each skill and each module. The target is highlighted where it is ahead. */
export function PlanProgressRows({ progress, server, labels, className }: IPlanProgressRowsProps): React.ReactElement {
    const { promotion, skills, modules } = progress;

    return (
        <div className={cn("flex flex-col gap-3 text-xs", className)}>
            <div className="flex items-center justify-between">
                <span className="font-medium text-muted-foreground">{labels.level}</span>
                <div className="flex items-center gap-2">
                    <div className="flex items-center gap-1 font-medium">
                        <img src={eliteIcon(promotion.current.elite)} alt={labels.eliteAlt(promotion.current.elite)} className="icon-theme-aware size-5 object-contain" />
                        <span>{labels.levelValue(promotion.current.level)}</span>
                    </div>
                    <Arrow />
                    <div className={cn("flex items-center gap-1 font-bold", upgradedTone(promotion.isUpgraded))}>
                        <img src={eliteIcon(promotion.target.elite)} alt={labels.eliteAlt(promotion.target.elite)} className={cn("icon-theme-aware size-5 object-contain", !promotion.isUpgraded && "opacity-50")} />
                        <span>{labels.levelValue(promotion.target.level)}</span>
                    </div>
                </div>
            </div>

            {skills.length > 0 && (
                <ProgressSection title={labels.skills}>
                    {skills.map(({ skill, index, current, target, isUpgraded }) => (
                        <ProgressRow
                            key={skill.skillId}
                            icon={<img src={skillIconURL(skill, server)} alt={skill.static?.levels?.[0]?.name} className="size-5 rounded border border-border/40 object-contain" />}
                            name={skill.static?.levels?.[0]?.name ?? labels.skillFallback(index + 1)}
                            current={labels.skillTarget(current)}
                            target={labels.skillTarget(target)}
                            isUpgraded={isUpgraded}
                        />
                    ))}
                </ProgressSection>
            )}

            {modules.length > 0 && (
                <ProgressSection title={labels.modules}>
                    {modules.map(({ module, current, target, isUpgraded }) => (
                        <ProgressRow key={module.uniEquipId} icon={<img src={moduleIconURL(module, server)} alt={module.uniEquipName} className="size-5 rounded object-contain" />} name={module.uniEquipName} current={labels.moduleStage(current)} target={labels.moduleStage(target)} isUpgraded={isUpgraded} />
                    ))}
                </ProgressSection>
            )}
        </div>
    );
}

function upgradedTone(isUpgraded: boolean): string {
    return isUpgraded ? "text-primary" : "text-muted-foreground";
}

function Arrow(): React.ReactElement {
    return <span className="text-muted-foreground/50">➔</span>;
}

function ProgressSection({ title, children }: { title: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div className="flex flex-col gap-2">
            <span className="font-medium text-muted-foreground">{title}</span>
            <div className="flex flex-col gap-1.5 pl-1">{children}</div>
        </div>
    );
}

interface IProgressRowProps {
    icon: React.ReactNode;
    name: string;
    current: string;
    target: string;
    isUpgraded: boolean;
}

function ProgressRow({ icon, name, current, target, isUpgraded }: IProgressRowProps): React.ReactElement {
    return (
        <div className="flex items-center justify-between">
            <div className="flex min-w-0 flex-1 items-center gap-1.5">
                {icon}
                <span className="truncate font-medium text-foreground">{name}</span>
            </div>
            <div className="ml-2 flex shrink-0 items-center gap-2">
                <span className="font-medium">{current}</span>
                <Arrow />
                <span className={cn("font-bold", upgradedTone(isUpgraded))}>{target}</span>
            </div>
        </div>
    );
}
