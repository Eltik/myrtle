"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "motion/react";
import Image from "next/image";
import type { Operator, EnrichedSkill, Talent } from "~/types/api";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Slider } from "~/components/ui/shadcn/slider";
import { cn } from "~/lib/utils";

interface SkillsTabProps {
    operator: Operator;
}

export function SkillsTab({ operator }: SkillsTabProps) {
    return (
        <div className="space-y-6">
            {/* Skills Section */}
            <InView
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
                transition={{ duration: 0.4 }}
            >
                <div className="rounded-xl border bg-card/50 p-6 backdrop-blur-sm">
                    <h2 className="mb-6 text-xl font-semibold">Skills</h2>
                    <div className="space-y-6">
                        {operator.skills.map((skill, index) => (
                            <SkillCard key={skill.skillId ?? index} skill={skill} index={index} />
                        ))}
                    </div>
                </div>
            </InView>

            {/* Talents Section */}
            <InView
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
                transition={{ duration: 0.4, delay: 0.1 }}
            >
                <div className="rounded-xl border bg-card/50 p-6 backdrop-blur-sm">
                    <h2 className="mb-6 text-xl font-semibold">Talents</h2>
                    <div className="space-y-4">
                        {operator.talents.map((talent, index) => (
                            <TalentCard key={index} talent={talent} index={index} />
                        ))}
                    </div>
                </div>
            </InView>
        </div>
    );
}

interface SkillCardProps {
    skill: EnrichedSkill;
    index: number;
}

function SkillCard({ skill, index }: SkillCardProps) {
    const [selectedLevel, setSelectedLevel] = useState(0);
    const levels = skill.static?.Levels ?? [];
    const currentLevel = levels[selectedLevel];
    const maxLevel = levels.length - 1;

    const skillIconUrl = skill.static?.IconId ? `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/skills/skill_icon_${skill.static.IconId}.png` : null;

    return (
        <motion.div initial={{ opacity: 0, x: -20 }} animate={{ opacity: 1, x: 0 }} transition={{ delay: index * 0.1 }} className="rounded-lg border bg-muted/30 p-4">
            <div className="flex items-start gap-4">
                {/* Skill icon */}
                <div className="relative h-16 w-16 flex-shrink-0 overflow-hidden rounded-lg bg-muted">{skillIconUrl && <Image src={skillIconUrl || "/placeholder.svg"} alt={currentLevel?.name ?? "Skill"} fill className="object-cover" unoptimized />}</div>

                {/* Skill info */}
                <div className="min-w-0 flex-1">
                    <div className="mb-2 flex items-center gap-2">
                        <h3 className="font-semibold">{currentLevel?.name ?? `Skill ${index + 1}`}</h3>
                        <span className="rounded-full bg-primary/20 px-2 py-0.5 text-xs text-primary">Lv. {selectedLevel + 1}</span>
                    </div>

                    {/* Level slider */}
                    {maxLevel > 0 && (
                        <div className="mb-3 flex items-center gap-3">
                            <span className="text-xs text-muted-foreground">1</span>
                            <Slider min={0} max={maxLevel} step={1} value={[selectedLevel]} onValueChange={(value) => setSelectedLevel(value[0] ?? 0)} className="flex-1" />
                            <span className="text-xs text-muted-foreground">{maxLevel + 1}</span>
                        </div>
                    )}

                    {/* SP info */}
                    {currentLevel?.spData && (
                        <div className="mb-3 flex flex-wrap gap-4 text-sm">
                            <div className="flex items-center gap-1">
                                <span className="text-muted-foreground">SP Cost:</span>
                                <span className="font-medium">{currentLevel.spData.SpCost}</span>
                            </div>
                            <div className="flex items-center gap-1">
                                <span className="text-muted-foreground">Initial SP:</span>
                                <span className="font-medium">{currentLevel.spData.InitSp}</span>
                            </div>
                            {currentLevel.duration && currentLevel.duration > 0 && (
                                <div className="flex items-center gap-1">
                                    <span className="text-muted-foreground">Duration:</span>
                                    <span className="font-medium">{currentLevel.duration}s</span>
                                </div>
                            )}
                        </div>
                    )}

                    {/* Description */}
                    <AnimatePresence mode="wait">
                        <motion.p key={selectedLevel} initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="text-sm leading-relaxed text-muted-foreground">
                            {currentLevel?.description ?? "No description available."}
                        </motion.p>
                    </AnimatePresence>
                </div>
            </div>
        </motion.div>
    );
}

interface TalentCardProps {
    talent: Talent;
    index: number;
}

function TalentCard({ talent, index }: TalentCardProps) {
    const [selectedCandidate, setSelectedCandidate] = useState((talent.Candidates?.length ?? 1) - 1);
    const candidates = talent.Candidates ?? [];
    const currentCandidate = candidates[selectedCandidate];

    return (
        <motion.div initial={{ opacity: 0, x: -20 }} animate={{ opacity: 1, x: 0 }} transition={{ delay: index * 0.1 }} className="rounded-lg border bg-muted/30 p-4">
            <div className="mb-2 flex items-center gap-2">
                <h3 className="font-semibold">{currentCandidate?.Name ?? `Talent ${index + 1}`}</h3>
                {candidates.length > 1 && (
                    <div className="flex gap-1">
                        {candidates.map((_, idx) => (
                            <button key={idx} onClick={() => setSelectedCandidate(idx)} className={cn("h-6 w-6 rounded text-xs transition-colors", selectedCandidate === idx ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground hover:text-foreground")}>
                                {idx + 1}
                            </button>
                        ))}
                    </div>
                )}
            </div>

            {currentCandidate?.UnlockCondition && (
                <p className="mb-2 text-xs text-muted-foreground">
                    Unlocks at Elite {currentCandidate.UnlockCondition.Phase?.replace("PHASE_", "")} Lv. {currentCandidate.UnlockCondition.Level}
                </p>
            )}

            <AnimatePresence mode="wait">
                <motion.p key={selectedCandidate} initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="text-sm leading-relaxed text-muted-foreground">
                    {currentCandidate?.Description ?? "No description available."}
                </motion.p>
            </AnimatePresence>
        </motion.div>
    );
}
