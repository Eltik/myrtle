"use client";

import { Award, ChevronDown, Zap } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import Image from "next/image";
import { useState } from "react";
import { AnimatedBackground } from "~/components/ui/motion-primitives/animated-background";
import { AnimatedGroup } from "~/components/ui/motion-primitives/animated-group";
import { Badge } from "~/components/ui/shadcn/badge";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/shadcn/collapsible";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Slider } from "~/components/ui/shadcn/slider";
import { descriptionToHtml } from "~/lib/description-parser";
import type { Operator, TalentCandidate } from "~/types/api";

interface SkillsTabProps {
    operator: Operator;
}

export function SkillsTab({ operator }: SkillsTabProps) {
    const [selectedSkillIndex, setSelectedSkillIndex] = useState(0);
    const [skillLevel, setSkillLevel] = useState(6); // 0-indexed, 6 = level 7
    const [isSkillsOpen, setIsSkillsOpen] = useState(true);
    const [isTalentsOpen, setIsTalentsOpen] = useState(true);

    // Talent display state
    const [selectedTalentCandidates, setSelectedTalentCandidates] = useState<Record<number, number>>({});

    const skills = operator.skills ?? [];
    const currentSkill = skills[selectedSkillIndex];
    const skillStatic = currentSkill?.static;
    const currentLevel = skillStatic?.Levels?.[skillLevel];

    const getSkillDescription = () => {
        if (!currentLevel?.description || !currentLevel.blackboard) return "";
        return descriptionToHtml(currentLevel.description, currentLevel.blackboard);
    };

    const getSpType = (spType: string) => {
        switch (spType) {
            case "INCREASE_WITH_TIME":
                return { label: "Auto Recovery", color: "#a7e855" };
            case "INCREASE_WHEN_ATTACK":
                return { label: "Offensive Recovery", color: "#f98d3f" };
            case "INCREASE_WHEN_TAKEN_DAMAGE":
                return { label: "Defensive Recovery", color: "#ffcf53" };
            case 8:
                return { label: "Passive", color: "#9c9a9a" };
            default:
                return { label: spType, color: "#9c9a9a" };
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

    // Get talent candidates for a specific talent index
    const getTalentCandidates = (talentIndex: number) => {
        const talent = operator.talents?.[talentIndex];
        return talent?.Candidates ?? [];
    };

    // Get selected candidate for a talent
    const _getSelectedCandidate = (talentIndex: number): TalentCandidate | null => {
        const candidates = getTalentCandidates(talentIndex);
        if (candidates.length === 0) return null;
        const selectedIndex = selectedTalentCandidates[talentIndex] ?? candidates.length - 1;
        return candidates[selectedIndex] ?? null;
    };

    // Format unlock condition
    const formatUnlockCondition = (candidate: TalentCandidate) => {
        const phase = candidate.UnlockCondition?.Phase;
        const level = candidate.UnlockCondition?.Level ?? 1;
        const potential = candidate.RequiredPotentialRank;

        const phaseStr = phase === "PHASE_0" ? "E0" : phase === "PHASE_1" ? "E1" : phase === "PHASE_2" ? "E2" : (phase?.replace("PHASE_", "E") ?? "E0");

        let result = `${phaseStr} Lv${level}`;
        if (potential > 0) {
            result += ` Pot${potential + 1}`;
        }
        return result;
    };

    const motionVariants = {
        open: { opacity: 1, height: "auto" },
        collapsed: { opacity: 0, height: 0 },
    };

    return (
        <div className="w-full space-y-6">
            <AnimatedGroup className="space-y-6" preset="blur-slide">
                {/* Skills Section */}
                <Collapsible defaultOpen={true} onOpenChange={setIsSkillsOpen} open={isSkillsOpen}>
                    <Card className="overflow-hidden">
                        <CollapsibleTrigger asChild className="w-full cursor-pointer">
                            <CardHeader className="flex w-full flex-row items-center justify-between space-y-0 bg-gradient-to-r from-card to-background/80 p-4 backdrop-blur-sm">
                                <CardTitle className="flex items-center gap-2">
                                    <Zap className="h-5 w-5 text-blue-400" />
                                    Skills
                                </CardTitle>
                                <ChevronDown className={`ml-auto h-6 w-6 transition-transform ${isSkillsOpen ? "rotate-180" : ""}`} />
                            </CardHeader>
                        </CollapsibleTrigger>
                        <AnimatePresence initial={false}>
                            {isSkillsOpen && (
                                <motion.div animate="open" exit="collapsed" initial="collapsed" key="skills-content" transition={{ duration: 0.3, ease: [0.04, 0.62, 0.23, 0.98] }} variants={motionVariants}>
                                    <CollapsibleContent forceMount>
                                        <CardContent className="p-4">
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

                                                    {/* Skill Level Selector with Slider */}
                                                    <div className="mt-4 space-y-3">
                                                        <div className="flex items-center justify-between">
                                                            <span className="font-medium text-sm">Skill Level</span>
                                                            <span className="text-muted-foreground text-xs">{skillLevel < 7 ? `Lv${skillLevel + 1}` : `M${skillLevel - 6}`}</span>
                                                        </div>
                                                        <div className="flex flex-col gap-3 md:flex-row md:items-center">
                                                            <Slider className="flex-1" max={(skillStatic?.Levels?.length ?? 10) - 1} min={0} onValueChange={(value) => setSkillLevel(value[0] ?? 0)} step={1} value={[skillLevel]} />
                                                            <Select onValueChange={(value) => setSkillLevel(Number.parseInt(value, 10))} value={String(skillLevel)}>
                                                                <SelectTrigger className="w-[130px]">
                                                                    <SelectValue />
                                                                </SelectTrigger>
                                                                <SelectContent>
                                                                    {Array.from({ length: skillStatic?.Levels?.length ?? 10 }).map((_, i) => (
                                                                        <SelectItem key={i} value={String(i)}>
                                                                            {i < 7 ? `Lv.${i + 1}` : `M${i - 6}`}
                                                                        </SelectItem>
                                                                    ))}
                                                                </SelectContent>
                                                            </Select>
                                                            {skillLevel >= 7 && <Image alt={`M${skillLevel - 6}`} className="h-8 w-9" height={50} src={`/m-${skillLevel - 6}_0.webp`} width={50} />}
                                                        </div>
                                                    </div>

                                                    {/* Skill Details */}
                                                    {currentLevel && (
                                                        <motion.div animate={{ opacity: 1, y: 0 }} className="mt-4 rounded-lg border border-border bg-card/50 p-4" initial={{ opacity: 0, y: 10 }} key={`${selectedSkillIndex}-${skillLevel}`}>
                                                            <div className="mb-4 flex items-center gap-3">
                                                                {skillStatic?.IconId && (
                                                                    <div className="relative h-14 w-14 overflow-hidden rounded-lg bg-muted">
                                                                        <Image alt={currentLevel.name} className="object-contain p-1" fill src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/skills/skill_icon_${skillStatic.IconId}.png`} />
                                                                    </div>
                                                                )}
                                                                <div>
                                                                    <h4 className="font-semibold text-lg">{currentLevel.name}</h4>
                                                                    <div className="flex flex-wrap gap-2 text-xs">
                                                                        <span className="rounded-md bg-muted px-2 py-0.5">{getSkillType(currentLevel.durationType ?? "")}</span>
                                                                        <span className="rounded-md bg-muted px-2 py-0.5 font-medium" style={{ color: getSpType(currentLevel.spData.spType).color }}>
                                                                            {getSpType(currentLevel.spData.spType).label}
                                                                        </span>
                                                                    </div>
                                                                </div>
                                                            </div>

                                                            {/* SP Stats */}
                                                            <div className="mb-4 grid grid-cols-2 gap-2 md:grid-cols-4">
                                                                <StatBox label="SP Cost" value={currentLevel.spData.spCost} />
                                                                <StatBox label="Initial SP" value={currentLevel.spData.initSp} />
                                                                <StatBox label="Duration" value={currentLevel.duration === 0 ? "Instant" : currentLevel.duration === -1 ? "Ammo" : `${currentLevel.duration}s`} />
                                                                <StatBox label="Skill Type" value={currentLevel.skillType ?? "Auto"} />
                                                            </div>

                                                            {/* Description with color formatting */}
                                                            <div className="rounded-md bg-muted/30 p-3">
                                                                <p
                                                                    className="text-sm leading-relaxed"
                                                                    dangerouslySetInnerHTML={{
                                                                        __html: getSkillDescription(),
                                                                    }}
                                                                />
                                                            </div>

                                                            {/* Stats Change (Blackboard) */}
                                                            {currentLevel.blackboard && currentLevel.blackboard.length > 0 && (
                                                                <div className="mt-4">
                                                                    <h5 className="mb-2 font-medium text-sm">Stats Change</h5>
                                                                    <div className="flex flex-wrap gap-2">
                                                                        {currentLevel.blackboard.map((bb) => (
                                                                            <Badge className="font-mono text-xs" key={bb.key} variant="secondary">
                                                                                {bb.key}: {formatBlackboardValue(bb.key, bb.value)}
                                                                            </Badge>
                                                                        ))}
                                                                    </div>
                                                                </div>
                                                            )}
                                                        </motion.div>
                                                    )}
                                                </>
                                            ) : (
                                                <p className="text-muted-foreground">This operator has no skills.</p>
                                            )}
                                        </CardContent>
                                    </CollapsibleContent>
                                </motion.div>
                            )}
                        </AnimatePresence>
                    </Card>
                </Collapsible>

                {/* Talents Section */}
                <Collapsible defaultOpen={true} onOpenChange={setIsTalentsOpen} open={isTalentsOpen}>
                    <Card className="overflow-hidden">
                        <CollapsibleTrigger asChild className="w-full cursor-pointer">
                            <CardHeader className="flex w-full flex-row items-center justify-between space-y-0 bg-gradient-to-r from-card to-background/80 p-4 backdrop-blur-sm">
                                <CardTitle className="flex items-center gap-2">
                                    <Award className="h-5 w-5 text-amber-500" />
                                    Talents
                                </CardTitle>
                                <ChevronDown className={`ml-auto h-6 w-6 transition-transform ${isTalentsOpen ? "rotate-180" : ""}`} />
                            </CardHeader>
                        </CollapsibleTrigger>
                        <AnimatePresence initial={false}>
                            {isTalentsOpen && (
                                <motion.div animate="open" exit="collapsed" initial="collapsed" key="talents-content" transition={{ duration: 0.3, ease: [0.04, 0.62, 0.23, 0.98] }} variants={motionVariants}>
                                    <CollapsibleContent forceMount>
                                        <CardContent className="p-4">
                                            {operator.talents && operator.talents.length > 0 ? (
                                                <div className="space-y-4">
                                                    {operator.talents.map((talent, talentIndex) => {
                                                        const candidates = talent.Candidates ?? [];
                                                        if (candidates.length === 0) return null;

                                                        const selectedCandidateIndex = selectedTalentCandidates[talentIndex] ?? candidates.length - 1;
                                                        const selectedCandidate = candidates[selectedCandidateIndex];

                                                        if (!selectedCandidate?.Name) return null;

                                                        const description = selectedCandidate.Blackboard ? descriptionToHtml(selectedCandidate.Description ?? "", selectedCandidate.Blackboard) : (selectedCandidate.Description ?? "");

                                                        return (
                                                            <div className="rounded-lg border border-border bg-gradient-to-r from-background to-background/80 p-4" key={talentIndex}>
                                                                <div className="mb-3 flex items-center justify-between">
                                                                    <Badge className="bg-secondary/50 font-semibold" variant="outline">
                                                                        Talent {talentIndex + 1}
                                                                    </Badge>

                                                                    {/* Candidate selector if multiple candidates */}
                                                                    {candidates.length > 1 && (
                                                                        <Select
                                                                            onValueChange={(value) =>
                                                                                setSelectedTalentCandidates((prev) => ({
                                                                                    ...prev,
                                                                                    [talentIndex]: Number.parseInt(value, 10),
                                                                                }))
                                                                            }
                                                                            value={String(selectedCandidateIndex)}
                                                                        >
                                                                            <SelectTrigger className="w-[180px]">
                                                                                <SelectValue />
                                                                            </SelectTrigger>
                                                                            <SelectContent>
                                                                                {candidates.map((candidate, candidateIndex) => (
                                                                                    <SelectItem key={candidateIndex} value={String(candidateIndex)}>
                                                                                        {formatUnlockCondition(candidate)}
                                                                                    </SelectItem>
                                                                                ))}
                                                                            </SelectContent>
                                                                        </Select>
                                                                    )}
                                                                </div>

                                                                <div className="rounded-lg bg-secondary/40 p-3">
                                                                    <div className="mb-2 flex items-center justify-between">
                                                                        <h3 className="font-semibold text-base">{selectedCandidate.Name}</h3>
                                                                        <div className="flex items-center gap-2">
                                                                            <Badge className="text-xs" variant="secondary">
                                                                                {formatUnlockCondition(selectedCandidate)}
                                                                            </Badge>
                                                                            {selectedCandidate.RequiredPotentialRank > 0 && (
                                                                                <Image alt={`Potential ${selectedCandidate.RequiredPotentialRank + 1}`} height={25} src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/potential/${selectedCandidate.RequiredPotentialRank + 1}.png`} width={25} />
                                                                            )}
                                                                        </div>
                                                                    </div>
                                                                    <div className="text-muted-foreground text-sm leading-relaxed" dangerouslySetInnerHTML={{ __html: description }} />

                                                                    {/* Blackboard values */}
                                                                    {selectedCandidate.Blackboard && selectedCandidate.Blackboard.length > 0 && (
                                                                        <div className="mt-3 flex flex-wrap gap-2">
                                                                            {selectedCandidate.Blackboard.map((bb) => (
                                                                                <Badge className="font-mono text-xs" key={bb.key} variant="secondary">
                                                                                    {bb.key}: {formatBlackboardValue(bb.key, bb.value)}
                                                                                </Badge>
                                                                            ))}
                                                                        </div>
                                                                    )}
                                                                </div>
                                                            </div>
                                                        );
                                                    })}
                                                </div>
                                            ) : (
                                                <p className="text-muted-foreground">This operator has no talents.</p>
                                            )}
                                        </CardContent>
                                    </CollapsibleContent>
                                </motion.div>
                            )}
                        </AnimatePresence>
                    </Card>
                </Collapsible>
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

function formatBlackboardValue(key: string, value: number): string {
    const percentageKeys = ["atk", "def", "attack@atk_scale", "hp_recovery_per_sec", "attack@atk_scale_ol", "attack@prob", "magic_resistance", "max_hp", "prob", "scale"];

    const lowerKey = (key ?? "").toLowerCase();

    if (percentageKeys.some((k) => lowerKey.includes(k))) {
        const percentValue = Math.round(value * 100);
        return `${value >= 0 ? "+" : ""}${percentValue}%`;
    }
    if (lowerKey.includes("duration") || lowerKey.includes("time")) {
        return `${value}s`;
    }
    return value > 0 ? `+${value}` : String(value);
}
