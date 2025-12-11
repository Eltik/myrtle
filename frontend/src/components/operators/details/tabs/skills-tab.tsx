"use client";

import { motion } from "motion/react";
import Image from "next/image";
import { useState } from "react";
import { AnimatedBackground } from "~/components/ui/motion-primitives/animated-background";
import { AnimatedGroup } from "~/components/ui/motion-primitives/animated-group";
import { Separator } from "~/components/ui/shadcn/separator";
import { insertBlackboard } from "~/lib/operator-helpers";
import type { Operator } from "~/types/api";

interface SkillsTabProps {
    operator: Operator;
}

export function SkillsTab({ operator }: SkillsTabProps) {
    const [selectedSkillIndex, setSelectedSkillIndex] = useState(0);
    const [skillLevel, setSkillLevel] = useState(6); // 0-indexed, 6 = level 7

    const skills = operator.skills ?? [];
    const currentSkill = skills[selectedSkillIndex];
    const skillStatic = currentSkill?.static;
    const currentLevel = skillStatic?.Levels?.[skillLevel];

    const getSkillDescription = () => {
        if (!currentLevel?.description || !currentLevel.blackboard) return "";
        return insertBlackboard(currentLevel.description, currentLevel.blackboard) ?? currentLevel.description;
    };

    const getSpType = (spType: string) => {
        switch (spType) {
            case "INCREASE_WITH_TIME":
                return "Auto Recovery";
            case "INCREASE_WHEN_ATTACK":
                return "Offensive Recovery";
            case "INCREASE_WHEN_TAKEN_DAMAGE":
                return "Defensive Recovery";
            default:
                return spType;
        }
    };

    const getSkillType = (durationType: string) => {
        switch (durationType) {
            case "NONE":
                return "Passive";
            default:
                return "Active";
        }
    };

    const handleSkillChange = (value: string | null) => {
        if (value) {
            const index = Number.parseInt(value.replace("skill-", ""), 10);
            if (!Number.isNaN(index)) {
                setSelectedSkillIndex(index);
            }
        }
    };

    return (
        <div className="w-full space-y-6">
            <div className="flex items-center justify-between">
                <h2 className="font-bold text-2xl md:text-3xl">Skills & Talents</h2>
            </div>
            <Separator />

            <AnimatedGroup className="space-y-6" preset="blur-slide">
                {/* Skills Section */}
                <div className="space-y-4">
                    <h3 className="font-semibold text-xl">Skills</h3>

                    {skills.length > 0 ? (
                        <>
                            {/* Skill Selector */}
                            <div className="flex flex-wrap gap-2">
                                <AnimatedBackground className="rounded-lg bg-primary" defaultValue={`skill-${selectedSkillIndex}`} onValueChange={handleSkillChange} transition={{ type: "spring", bounce: 0.2, duration: 0.4 }}>
                                    {skills.map((skill, index) => (
                                        <button className="relative z-10 flex items-center gap-2 rounded-lg px-4 py-2 transition-colors data-[checked=true]:text-primary-foreground" data-id={`skill-${index}`} key={skill.skillId} type="button">
                                            {skill.static?.IconId && (
                                                <div className="relative h-8 w-8 overflow-hidden rounded-md bg-muted">
                                                    <Image alt={skill.static?.Levels?.[0]?.name ?? "Skill"} className="object-contain p-0.5" fill src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/skills/skill_icon_${skill.static.IconId}.png`} />
                                                </div>
                                            )}
                                            <span className="font-medium text-sm">{skill.static?.Levels?.[0]?.name ?? `Skill ${index + 1}`}</span>
                                        </button>
                                    ))}
                                </AnimatedBackground>
                            </div>

                            {/* Skill Level Selector */}
                            <div className="flex flex-wrap gap-1">
                                {Array.from({ length: 10 }).map((_, i) => {
                                    const isAvailable = skillStatic?.Levels?.[i];
                                    const label = i < 7 ? `Lv${i + 1}` : `M${i - 6}`;
                                    return (
                                        <button
                                            className={`rounded-md px-3 py-1.5 font-medium text-xs transition-colors ${skillLevel === i ? "bg-primary text-primary-foreground" : isAvailable ? "bg-muted hover:bg-muted/80" : "cursor-not-allowed bg-muted/30 text-muted-foreground/50"}`}
                                            disabled={!isAvailable}
                                            key={i}
                                            onClick={() => isAvailable && setSkillLevel(i)}
                                        >
                                            {label}
                                        </button>
                                    );
                                })}
                            </div>

                            {/* Skill Details */}
                            {currentLevel && (
                                <motion.div animate={{ opacity: 1, y: 0 }} className="rounded-lg border border-border bg-card/50 p-4" initial={{ opacity: 0, y: 10 }} key={`${selectedSkillIndex}-${skillLevel}`}>
                                    <div className="mb-4 flex items-center gap-3">
                                        {skillStatic?.IconId && (
                                            <div className="relative h-14 w-14 overflow-hidden rounded-lg bg-muted">
                                                <Image alt={currentLevel.name} className="object-contain p-1" fill src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/skills/skill_icon_${skillStatic.IconId}.png`} />
                                            </div>
                                        )}
                                        <div>
                                            <h4 className="font-semibold text-lg">{currentLevel.name}</h4>
                                            <div className="flex flex-wrap gap-2 text-muted-foreground text-xs">
                                                <span className="rounded-md bg-muted px-2 py-0.5">{getSkillType(currentLevel.durationType)}</span>
                                                <span className="rounded-md bg-muted px-2 py-0.5">{getSpType(currentLevel.spData.spType)}</span>
                                            </div>
                                        </div>
                                    </div>

                                    {/* SP Stats */}
                                    <div className="mb-4 grid grid-cols-3 gap-2">
                                        <StatBox label="SP Cost" value={currentLevel.spData.spCost} />
                                        <StatBox label="Initial SP" value={currentLevel.spData.initSp} />
                                        <StatBox label="Duration" value={currentLevel.duration > 0 ? `${currentLevel.duration}s` : "-"} />
                                    </div>

                                    {/* Description */}
                                    <div className="rounded-md bg-muted/30 p-3">
                                        <p
                                            className="text-sm leading-relaxed"
                                            dangerouslySetInnerHTML={{
                                                __html: getSkillDescription(),
                                            }}
                                        />
                                    </div>
                                </motion.div>
                            )}
                        </>
                    ) : (
                        <p className="text-muted-foreground">This operator has no skills.</p>
                    )}
                </div>

                {/* Talents Section */}
                <div className="space-y-4">
                    <h3 className="font-semibold text-xl">Talents</h3>
                    {operator.talents && operator.talents.length > 0 ? (
                        <div className="space-y-3">
                            {operator.talents.map((talent, index) => {
                                const candidate = talent.Candidates?.[talent.Candidates.length - 1];
                                if (!candidate?.Name) return null;

                                const description = candidate.Blackboard ? insertBlackboard(candidate.Description ?? "", candidate.Blackboard) : candidate.Description;

                                return (
                                    <motion.div animate={{ opacity: 1, y: 0 }} className="rounded-lg border border-border bg-card/50 p-4" initial={{ opacity: 0, y: 10 }} key={index} transition={{ delay: index * 0.1 }}>
                                        <h4 className="mb-2 font-semibold text-primary">{candidate.Name}</h4>
                                        <p
                                            className="text-muted-foreground text-sm leading-relaxed"
                                            dangerouslySetInnerHTML={{
                                                __html: description ?? "",
                                            }}
                                        />
                                    </motion.div>
                                );
                            })}
                        </div>
                    ) : (
                        <p className="text-muted-foreground">This operator has no talents.</p>
                    )}
                </div>
            </AnimatedGroup>
        </div>
    );
}

function StatBox({ label, value }: { label: string; value: number | string }) {
    return (
        <div className="rounded-md bg-muted/50 p-2 text-center">
            <div className="text-muted-foreground text-xs">{label}</div>
            <div className="font-semibold">{value}</div>
        </div>
    );
}
