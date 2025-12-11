"use client";

import { motion } from "motion/react";
import Image from "next/image";
import { useEffect, useState } from "react";
import { AnimatedGroup } from "~/components/ui/motion-primitives/animated-group";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Separator } from "~/components/ui/shadcn/separator";
import { Slider } from "~/components/ui/shadcn/slider";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";
import type { Item, Operator } from "~/types/api";

interface LevelCostTabProps {
    operator: Operator;
}

interface MaterialCost {
    id: string;
    count: number;
}

export function LevelCostTab({ operator }: LevelCostTabProps) {
    const [materials, setMaterials] = useState<Record<string, Item>>({});
    const [activeTab, setActiveTab] = useState("elite");

    // Level-up cost state
    const [selectedElite, setSelectedElite] = useState(0);
    const [selectedSkillLevel, setSelectedSkillLevel] = useState(1); // 1-indexed: 2-7 for normal, 8-10 for mastery
    const [selectedSkillIndex, setSelectedSkillIndex] = useState(0);
    const [selectedModuleIndex, setSelectedModuleIndex] = useState(0);
    const [selectedModuleStage, setSelectedModuleStage] = useState(1);

    // Fetch materials
    useEffect(() => {
        const fetchMaterials = async () => {
            try {
                const response = await fetch("/api/static", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ type: "materials" }),
                });
                const data = (await response.json()) as { data: Item[] };
                const materialsMap: Record<string, Item> = {};
                for (const item of data.data ?? []) {
                    if (item.itemId) {
                        materialsMap[item.itemId] = item;
                    }
                }
                setMaterials(materialsMap);
            } catch (error) {
                console.error("Failed to fetch materials:", error);
            }
        };
        void fetchMaterials();
    }, []);

    // Elite promotion costs
    const eliteCosts = operator.phases
        .filter((phase, index) => index > 0 && phase.EvolveCost)
        .map((phase, index) => ({
            elite: index + 1,
            label: `E${index + 1}`,
            costs: phase.EvolveCost ?? [],
        }));

    // Skill level costs (shared across all skills for levels 2-7)
    const skillLevelCosts = operator.allSkillLevelUp ?? [];

    // Skill mastery costs (per skill)
    const masterySkills = operator.skills?.filter((s) => s.levelUpCostCond && s.levelUpCostCond.length > 0);

    // Module costs
    const modulesWithCosts = operator.modules?.filter((m) => m.type !== "INITIAL" && m.itemCost) ?? [];

    return (
        <div className="w-full space-y-6">
            <div className="flex items-center justify-between">
                <h2 className="font-bold text-2xl md:text-3xl">Level-Up Cost</h2>
            </div>
            <Separator />

            <Tabs onValueChange={setActiveTab} value={activeTab}>
                <TabsList className="grid w-full grid-cols-3">
                    <TabsTrigger value="elite">Elite Promotion</TabsTrigger>
                    <TabsTrigger value="skill">Skill Level Up</TabsTrigger>
                    <TabsTrigger value="module">Module</TabsTrigger>
                </TabsList>

                {/* Elite Promotion Tab */}
                <TabsContent className="mt-4" value="elite">
                    <AnimatedGroup className="space-y-4" preset="blur-slide">
                        {eliteCosts.length > 0 ? (
                            <>
                                {/* Elite Selector */}
                                <div className="space-y-3 rounded-lg border border-border bg-card/50 p-4">
                                    <div className="flex items-center justify-between">
                                        <label className="font-medium text-sm">Select Promotion</label>
                                        <span className="text-muted-foreground text-xs">E{selectedElite + 1}</span>
                                    </div>
                                    <Slider max={eliteCosts.length - 1} min={0} onValueChange={(value) => setSelectedElite(value[0] ?? 0)} step={1} value={[selectedElite]} />
                                    <div className="flex gap-2">
                                        {eliteCosts.map((elite, index) => (
                                            <button className={`flex-1 rounded-md py-2 text-sm transition-colors ${selectedElite === index ? "bg-primary text-primary-foreground" : "bg-muted hover:bg-muted/80"}`} key={elite.label} onClick={() => setSelectedElite(index)}>
                                                {elite.label}
                                            </button>
                                        ))}
                                    </div>
                                </div>

                                {/* Materials Display */}
                                <motion.div animate={{ opacity: 1, y: 0 }} className="rounded-lg border border-border bg-card/50 p-4" initial={{ opacity: 0, y: 10 }} key={selectedElite}>
                                    <h4 className="mb-3 font-semibold text-primary">{eliteCosts[selectedElite]?.label} Promotion</h4>
                                    <div className="flex flex-wrap gap-2">
                                        {eliteCosts[selectedElite]?.costs.map((cost: MaterialCost, i: number) => (
                                            <MaterialItem count={cost.count} key={i} material={materials[cost.id]} />
                                        ))}
                                    </div>
                                </motion.div>
                            </>
                        ) : (
                            <p className="text-muted-foreground">This operator cannot be promoted.</p>
                        )}
                    </AnimatedGroup>
                </TabsContent>

                {/* Skill Level Up Tab */}
                <TabsContent className="mt-4" value="skill">
                    <AnimatedGroup className="space-y-4" preset="blur-slide">
                        {/* Skill Level Selector */}
                        <div className="space-y-4 rounded-lg border border-border bg-card/50 p-4">
                            <div className="flex items-center justify-between">
                                <label className="font-medium text-sm">Skill Level</label>
                                <span className="text-muted-foreground text-xs">{selectedSkillLevel <= 6 ? `Lv${selectedSkillLevel + 1}` : `M${selectedSkillLevel - 6}`}</span>
                            </div>
                            <Slider max={masterySkills && masterySkills.length > 0 ? 9 : 5} min={0} onValueChange={(value) => setSelectedSkillLevel(value[0] ?? 0)} step={1} value={[selectedSkillLevel]} />
                            <div className="flex flex-wrap gap-1">
                                {/* Levels 2-7 */}
                                {Array.from({ length: 6 }).map((_, i) => (
                                    <button className={`rounded-md px-3 py-1.5 text-xs transition-colors ${selectedSkillLevel === i ? "bg-primary text-primary-foreground" : "bg-muted hover:bg-muted/80"}`} key={i} onClick={() => setSelectedSkillLevel(i)}>
                                        Lv{i + 2}
                                    </button>
                                ))}
                                {/* Mastery levels */}
                                {masterySkills &&
                                    masterySkills.length > 0 &&
                                    [7, 8, 9].map((lvl) => (
                                        <button className={`rounded-md px-3 py-1.5 text-xs transition-colors ${selectedSkillLevel === lvl - 1 ? "bg-primary text-primary-foreground" : "bg-muted hover:bg-muted/80"}`} key={lvl} onClick={() => setSelectedSkillLevel(lvl - 1)}>
                                            M{lvl - 6}
                                        </button>
                                    ))}
                            </div>

                            {/* Skill selector for mastery */}
                            {selectedSkillLevel >= 6 && masterySkills && masterySkills.length > 0 && (
                                <div className="mt-3">
                                    <label className="mb-2 block font-medium text-sm">Select Skill</label>
                                    <Select onValueChange={(value) => setSelectedSkillIndex(Number.parseInt(value, 10))} value={String(selectedSkillIndex)}>
                                        <SelectTrigger>
                                            <SelectValue />
                                        </SelectTrigger>
                                        <SelectContent>
                                            {masterySkills.map((skill, index) => (
                                                <SelectItem key={skill.skillId} value={String(index)}>
                                                    {skill.static?.Levels?.[0]?.name ?? `Skill ${index + 1}`}
                                                </SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                </div>
                            )}
                        </div>

                        {/* Materials Display */}
                        <motion.div animate={{ opacity: 1, y: 0 }} className="rounded-lg border border-border bg-card/50 p-4" initial={{ opacity: 0, y: 10 }} key={`${selectedSkillLevel}-${selectedSkillIndex}`}>
                            <h4 className="mb-3 font-semibold text-primary">{selectedSkillLevel <= 5 ? `Level ${selectedSkillLevel + 2}` : `${masterySkills?.[selectedSkillIndex]?.static?.Levels?.[0]?.name ?? "Skill"} M${selectedSkillLevel - 5}`}</h4>
                            <div className="flex flex-wrap gap-2">
                                {selectedSkillLevel <= 5
                                    ? // Regular skill levels 2-7
                                      skillLevelCosts[selectedSkillLevel]?.LvlUpCost?.map((cost: MaterialCost, i: number) => <MaterialItem count={cost.count} key={i} material={materials[cost.id]} />)
                                    : // Mastery levels
                                      masterySkills?.[selectedSkillIndex]?.levelUpCostCond?.[selectedSkillLevel - 6]?.LevelUpCost?.map((cost: MaterialCost, i: number) => <MaterialItem count={cost.count} key={i} material={materials[cost.id]} />)}
                            </div>
                        </motion.div>

                        {skillLevelCosts.length === 0 && (!masterySkills || masterySkills.length === 0) && <p className="text-muted-foreground">No skill upgrade costs available.</p>}
                    </AnimatedGroup>
                </TabsContent>

                {/* Module Tab */}
                <TabsContent className="mt-4" value="module">
                    <AnimatedGroup className="space-y-4" preset="blur-slide">
                        {modulesWithCosts.length > 0 ? (
                            <>
                                {/* Module Selector */}
                                <div className="space-y-4 rounded-lg border border-border bg-card/50 p-4">
                                    <div>
                                        <label className="mb-2 block font-medium text-sm">Select Module</label>
                                        <Select onValueChange={(value) => setSelectedModuleIndex(Number.parseInt(value, 10))} value={String(selectedModuleIndex)}>
                                            <SelectTrigger>
                                                <SelectValue />
                                            </SelectTrigger>
                                            <SelectContent>
                                                {modulesWithCosts.map((module, index) => (
                                                    <SelectItem key={module.uniEquipId} value={String(index)}>
                                                        {module.typeName1 && module.typeName2 ? `${module.typeName1}-${module.typeName2}` : module.uniEquipName}
                                                    </SelectItem>
                                                ))}
                                            </SelectContent>
                                        </Select>
                                    </div>

                                    {/* Stage Selector */}
                                    <div>
                                        <div className="mb-2 flex items-center justify-between">
                                            <label className="font-medium text-sm">Module Stage</label>
                                            <span className="text-muted-foreground text-xs">Stage {selectedModuleStage}</span>
                                        </div>
                                        <Slider max={3} min={1} onValueChange={(value) => setSelectedModuleStage(value[0] ?? 1)} step={1} value={[selectedModuleStage]} />
                                        <div className="mt-2 flex gap-2">
                                            {[1, 2, 3].map((stage) => (
                                                <button className={`flex-1 rounded-md py-2 text-sm transition-colors ${selectedModuleStage === stage ? "bg-primary text-primary-foreground" : "bg-muted hover:bg-muted/80"}`} key={stage} onClick={() => setSelectedModuleStage(stage)}>
                                                    Stage {stage}
                                                </button>
                                            ))}
                                        </div>
                                    </div>
                                </div>

                                {/* Materials Display */}
                                <motion.div animate={{ opacity: 1, y: 0 }} className="rounded-lg border border-border bg-card/50 p-4" initial={{ opacity: 0, y: 10 }} key={`${selectedModuleIndex}-${selectedModuleStage}`}>
                                    <h4 className="mb-3 font-semibold text-primary">
                                        {modulesWithCosts[selectedModuleIndex]?.uniEquipName} - Stage {selectedModuleStage}
                                    </h4>
                                    <div className="flex flex-wrap gap-2">
                                        {(() => {
                                            const moduleRef = modulesWithCosts[selectedModuleIndex];
                                            const stageCosts = moduleRef?.itemCost?.[String(selectedModuleStage)];
                                            if (!stageCosts) return <p className="text-muted-foreground">No costs for this stage.</p>;

                                            if (Array.isArray(stageCosts)) {
                                                return stageCosts.map((cost: MaterialCost, i: number) => <MaterialItem count={cost.count} key={i} material={materials[cost.id]} />);
                                            }
                                            return <MaterialItem count={(stageCosts as MaterialCost).count} material={materials[(stageCosts as MaterialCost).id]} />;
                                        })()}
                                    </div>
                                </motion.div>
                            </>
                        ) : (
                            <p className="text-muted-foreground">This operator has no modules.</p>
                        )}
                    </AnimatedGroup>
                </TabsContent>
            </Tabs>
        </div>
    );
}

function MaterialItem({ material, count }: { material?: Item; count: number }) {
    const iconUrl = material?.iconId ? `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/items/${material.iconId}.png` : "/abstract-material.png";

    return (
        <div className="flex items-center gap-2 rounded-md bg-muted/50 px-3 py-2 transition-colors hover:bg-muted/70">
            <div className="relative h-10 w-10">
                <Image alt={material?.name ?? "Material"} className="object-contain" fill src={iconUrl || "/placeholder.svg"} />
            </div>
            <div className="flex flex-col">
                <span className="max-w-[120px] truncate text-xs">{material?.name ?? "Unknown"}</span>
                <span className="font-bold text-sm">x{count}</span>
            </div>
        </div>
    );
}
