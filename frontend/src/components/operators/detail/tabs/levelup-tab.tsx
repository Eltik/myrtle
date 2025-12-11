"use client";

import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "motion/react";
import Image from "next/image";
import type { Operator, Item } from "~/types/api";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { cn } from "~/lib/utils";

interface LevelUpTabProps {
    operator: Operator;
}

type CostTab = "elite" | "skill" | "module";

export function LevelUpTab({ operator }: LevelUpTabProps) {
    const [activeTab, setActiveTab] = useState<CostTab>("elite");
    const [materials, setMaterials] = useState<Record<string, Item>>({});

    // Fetch materials
    useEffect(() => {
        const fetchMaterials = async () => {
            try {
                const response = await fetch("/api/static", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ type: "materials" }),
                });
                if (response.ok) {
                    const data = (await response.json()) as { data: Item[] };
                    // Convert array to Record for easy lookup by itemId
                    const materialsMap: Record<string, Item> = {};
                    for (const item of data.data ?? []) {
                        materialsMap[item.itemId] = item;
                    }
                    setMaterials(materialsMap);
                }
            } catch (error) {
                console.error("Failed to fetch materials:", error);
            }
        };
        void fetchMaterials();
    }, []);

    const tabs: { id: CostTab; label: string }[] = [
        { id: "elite", label: "Elite Promotion" },
        { id: "skill", label: "Skill Level Up" },
        { id: "module", label: "Module" },
    ];

    return (
        <div className="space-y-6">
            <InView
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
                transition={{ duration: 0.4 }}
            >
                <div className="rounded-xl border bg-card/50 p-6 backdrop-blur-sm">
                    <h2 className="mb-6 text-xl font-semibold">Level-Up Costs</h2>

                    {/* Tab buttons */}
                    <div className="mb-6 flex gap-2 rounded-lg bg-muted p-1">
                        {tabs.map((tab) => (
                            <button key={tab.id} onClick={() => setActiveTab(tab.id)} className={cn("relative flex-1 rounded-md px-4 py-2 text-sm font-medium transition-colors", activeTab === tab.id ? "text-foreground" : "text-muted-foreground hover:text-foreground")}>
                                {activeTab === tab.id && <motion.div layoutId="costTab" className="absolute inset-0 rounded-md bg-card shadow-sm" transition={{ type: "spring", bounce: 0.2, duration: 0.6 }} />}
                                <span className="relative z-10">{tab.label}</span>
                            </button>
                        ))}
                    </div>

                    {/* Tab content */}
                    <AnimatePresence mode="wait">
                        <motion.div key={activeTab} initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -10 }} transition={{ duration: 0.2 }}>
                            {activeTab === "elite" && <ElitePromotionContent operator={operator} materials={materials} />}
                            {activeTab === "skill" && <SkillLevelContent operator={operator} materials={materials} />}
                            {activeTab === "module" && <ModuleLevelContent operator={operator} materials={materials} />}
                        </motion.div>
                    </AnimatePresence>
                </div>
            </InView>
        </div>
    );
}

interface ContentProps {
    operator: Operator;
    materials: Record<string, Item>;
}

function ElitePromotionContent({ operator, materials }: ContentProps) {
    const promotions = operator.phases
        .slice(1) // Skip E0
        .map((phase, index) => ({
            elite: `E${index + 1}`,
            costs: phase.EvolveCost ?? [],
        }));

    if (promotions.length === 0 || promotions.every((p) => p.costs.length === 0)) {
        return <p className="text-sm text-muted-foreground">No promotion costs available for this operator.</p>;
    }

    return (
        <div className="space-y-6">
            {promotions.map(
                (promo) =>
                    promo.costs.length > 0 && (
                        <div key={promo.elite} className="rounded-lg border bg-muted/30 p-4">
                            <h3 className="mb-3 font-semibold">{promo.elite}</h3>
                            <div className="flex flex-wrap gap-3">
                                {promo.costs.map((cost, idx) => (
                                    <MaterialItem key={idx} itemId={cost.Id} count={cost.Count} materials={materials} />
                                ))}
                            </div>
                        </div>
                    ),
            )}
        </div>
    );
}

function SkillLevelContent({ operator, materials }: ContentProps) {
    const [selectedSkill, setSelectedSkill] = useState(0);

    // Regular skill level costs (shared across all skills)
    const regularCosts = operator.allSkillLevelUp ?? [];

    // Mastery costs for the selected skill
    const selectedSkillData = operator.skills[selectedSkill];
    const masteryCosts = selectedSkillData?.levelUpCostCond ?? [];

    return (
        <div className="space-y-6">
            {/* Skill selector */}
            {operator.skills.length > 1 && (
                <div className="flex flex-wrap gap-2">
                    {operator.skills.map((skill, idx) => (
                        <button key={skill.skillId ?? idx} onClick={() => setSelectedSkill(idx)} className={cn("rounded-lg px-3 py-1.5 text-sm transition-colors", selectedSkill === idx ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground hover:text-foreground")}>
                            Skill {idx + 1}
                        </button>
                    ))}
                </div>
            )}

            {/* Skill levels 2-7 */}
            <div>
                <h4 className="mb-3 text-sm font-medium text-muted-foreground">Skill Levels 2-7</h4>
                <div className="space-y-3">
                    {regularCosts.map((cost, idx) => (
                        <div key={idx} className="rounded-lg border bg-muted/30 p-3">
                            <span className="mb-2 block text-sm font-medium">Level {idx + 2}</span>
                            <div className="flex flex-wrap gap-2">
                                {cost.LvlUpCost?.map((item, itemIdx) => (
                                    <MaterialItem key={itemIdx} itemId={item.Id} count={item.Count} materials={materials} />
                                ))}
                            </div>
                        </div>
                    ))}
                </div>
            </div>

            {/* Mastery levels */}
            {masteryCosts.length > 0 && (
                <div>
                    <h4 className="mb-3 text-sm font-medium text-muted-foreground">Mastery Levels</h4>
                    <div className="space-y-3">
                        {masteryCosts.map((cost, idx) => (
                            <div key={idx} className="rounded-lg border bg-muted/30 p-3">
                                <span className="mb-2 block text-sm font-medium">M{idx + 1}</span>
                                <div className="flex flex-wrap gap-2">
                                    {cost.LevelUpCost?.map((item, itemIdx) => (
                                        <MaterialItem key={itemIdx} itemId={item.Id} count={item.Count} materials={materials} />
                                    ))}
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}

function ModuleLevelContent({ operator, materials }: ContentProps) {
    const [selectedModule, setSelectedModule] = useState(0);

    if (!operator.modules || operator.modules.length === 0) {
        return <p className="text-sm text-muted-foreground">No modules available for this operator.</p>;
    }

    const module = operator.modules[selectedModule];
    const itemCost = module?.itemCost;

    return (
        <div className="space-y-6">
            {/* Module selector */}
            {operator.modules.length > 1 && (
                <div className="flex flex-wrap gap-2">
                    {operator.modules.map((mod, idx) => (
                        <button key={mod.uniEquipId ?? idx} onClick={() => setSelectedModule(idx)} className={cn("rounded-lg px-3 py-1.5 text-sm transition-colors", selectedModule === idx ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground hover:text-foreground")}>
                            {mod.uniEquipName ?? `Module ${idx + 1}`}
                        </button>
                    ))}
                </div>
            )}

            {/* Module stages */}
            {itemCost && (
                <div className="space-y-3">
                    {["1", "2", "3"].map((stage) => {
                        const stageCosts = itemCost[stage];
                        if (!stageCosts || !Array.isArray(stageCosts) || stageCosts.length === 0) {
                            return null;
                        }
                        return (
                            <div key={stage} className="rounded-lg border bg-muted/30 p-3">
                                <span className="mb-2 block text-sm font-medium">Stage {stage}</span>
                                <div className="flex flex-wrap gap-2">
                                    {stageCosts.map((cost: { id: string; count: number }, idx: number) => (
                                        <MaterialItem key={idx} itemId={cost.id} count={cost.count} materials={materials} />
                                    ))}
                                </div>
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
}

interface MaterialItemProps {
    itemId: string;
    count: number;
    materials: Record<string, Item>;
}

function MaterialItem({ itemId, count, materials }: MaterialItemProps) {
    const material = materials[itemId];
    const name = material?.name ?? itemId;
    const iconUrl = material?.iconId ? `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/items/${material.iconId}.png` : null;

    return (
        <div className="flex items-center gap-2 rounded-md bg-background/50 px-2 py-1">
            {iconUrl && (
                <div className="relative h-8 w-8 overflow-hidden rounded">
                    <Image src={iconUrl || "/placeholder.svg"} alt={name} fill className="object-contain" unoptimized />
                </div>
            )}
            <div className="text-sm">
                <p className="font-medium">{name}</p>
                <p className="text-xs text-muted-foreground">x{count}</p>
            </div>
        </div>
    );
}
