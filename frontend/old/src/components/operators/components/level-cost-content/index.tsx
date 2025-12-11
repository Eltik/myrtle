import { useEffect, useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import { Separator } from "~/components/ui/separator";
import type { Operator } from "~/types/impl/api/static/operator";
import type { Item } from "~/types/impl/api/static/material";
import type { MaterialCost, SkillLevelCost } from "~/types/impl/frontend/impl/operators";
import { ElitePromotionTab } from "./impl/elite-promotion-tab";
import { SkillLevelUpTab } from "./impl/skill-level-up-tab";
import { ModuleTab } from "./impl/module-tab";

function LevelUpContent({ operator }: { operator: Operator }) {
    const [materials, setMaterials] = useState<Item[]>([]);
    const [activeTab, setActiveTab] = useState<"elite" | "skill" | "module">("elite");
    const [elitePromotionCosts, setElitePromotionCosts] = useState<
        {
            elite: string;
            materials: MaterialCost[];
        }[]
    >([]);
    const [skillLevelCosts, setSkillLevelCosts] = useState<Record<string, SkillLevelCost[]>>({});
    const [moduleCosts, setModuleCosts] = useState<Record<string, MaterialCost[][]>>({});

    // Separate useEffect for setting up costs based on operator
    useEffect(() => {
        const elitePromCosts = operator.phases
            .filter((phase, index) => index > 0 && phase.evolveCost) // Skip first phase (E0) as it has no promotion cost
            .map((phase, index) => {
                return {
                    elite: `E${index + 1}`,
                    materials:
                        phase.evolveCost?.map((cost) => ({
                            quantity: cost.count,
                            material: {
                                itemId: cost.id,
                                name: cost.id,
                            },
                        })) ?? [],
                };
            });

        setElitePromotionCosts(elitePromCosts);

        // Process skill level costs for each skill
        const skillCosts: Record<string, SkillLevelCost[]> = {};

        // First add the regular skill level costs (1-7) which are shared across all skills
        const regularSkillCosts =
            operator.allSkillLvlup?.map((cost, index) => ({
                level: index + 2,
                phase: cost.unlockCond.phase,
                materials:
                    cost.lvlUpCost?.map((material) => ({
                        quantity: material.count,
                        material: {
                            itemId: material.id,
                            name: material.id,
                        },
                    })) ?? [],
            })) ?? [];

        // Then add mastery costs for each skill
        operator.skills?.forEach((skill) => {
            if (skill.skillId) {
                // Initialize with regular skill costs (1-7)
                skillCosts[skill.skillId] = [...regularSkillCosts];

                // Add mastery costs if they exist
                const masteryCosts =
                    skill.levelUpCostCond?.map((costCond, index) => ({
                        level: 7 + (index + 1),
                        phase: costCond.unlockCond.phase,
                        materials:
                            costCond.levelUpCost?.map((cost) => ({
                                quantity: cost.count,
                                material: {
                                    itemId: cost.id,
                                    name: cost.id,
                                },
                            })) ?? [],
                    })) ?? [];

                // We know this exists since we just initialized it above
                skillCosts[skill.skillId]!.push(...masteryCosts);
            }
        });

        setSkillLevelCosts(skillCosts);

        // Process module costs
        const modCosts: Record<string, MaterialCost[][]> = {};

        // Populate module costs for each module
        operator.modules?.forEach((moduleData) => {
            if (moduleData.uniEquipId) {
                const costs: MaterialCost[][] = [];

                // Each module has multiple levels (stages 1-3)
                for (let i = 1; i <= 3; i++) {
                    const stageCosts: MaterialCost[] = [];
                    const stageKey = `${i}`;

                    if (moduleData.itemCost?.[stageKey]) {
                        // itemCost has keys "1", "2", "3" for each stage
                        // Each stage has an array of materials
                        if (Array.isArray(moduleData.itemCost[stageKey])) {
                            // Process each material in the array for this stage
                            moduleData.itemCost[stageKey].forEach((material: { id: string; count: number; type: string }) => {
                                stageCosts.push({
                                    quantity: material.count,
                                    material: {
                                        itemId: material.id,
                                        name: material.id,
                                    },
                                });
                            });
                        } else {
                            // Handle the case where it's a single object (not array)
                            const material = moduleData.itemCost[stageKey] as { id: string; count: number; type: string };
                            stageCosts.push({
                                quantity: material.count,
                                material: {
                                    itemId: material.id,
                                    name: material.id,
                                },
                            });
                        }
                    }

                    costs.push(stageCosts);
                }

                modCosts[moduleData.uniEquipId] = costs;
            }
        });

        setModuleCosts(modCosts);
    }, [operator]);

    // Separate useEffect for fetching materials
    useEffect(() => {
        async function loadMaterials() {
            try {
                const data = (await (
                    await fetch("/api/static", {
                        method: "POST",
                        headers: {
                            "Content-Type": "application/json",
                        },
                        body: JSON.stringify({
                            type: "materials",
                        }),
                    })
                ).json()) as { data: { items: Record<string, Item> } };

                setMaterials(Object.values(data.data.items));
            } catch (error) {
                console.error("Failed to fetch materials:", error);
            }
        }

        void loadMaterials();
    }, []);

    return (
        <>
            <div className="p-2 px-4 backdrop-blur-2xl">
                <span className="text-xl font-bold md:text-3xl">Level-Up Cost</span>
            </div>
            <Separator />
            <div className="p-4">
                <Tabs value={activeTab} onValueChange={(value) => setActiveTab(value as "elite" | "skill" | "module")}>
                    <TabsList className="grid w-full grid-cols-3">
                        <TabsTrigger value="elite">Elite Promotion</TabsTrigger>
                        <TabsTrigger value="skill">Skill Level Up</TabsTrigger>
                        <TabsTrigger value="module">Module</TabsTrigger>
                    </TabsList>

                    <TabsContent value="elite" className="mt-4">
                        <ElitePromotionTab elitePromotionCosts={elitePromotionCosts} materials={materials} />
                    </TabsContent>

                    <TabsContent value="skill" className="mt-4">
                        <SkillLevelUpTab operator={operator} skillLevelCosts={skillLevelCosts} materials={materials} />
                    </TabsContent>

                    <TabsContent value="module" className="mt-4">
                        <ModuleTab operator={operator} moduleCosts={moduleCosts} materials={materials} />
                    </TabsContent>
                </Tabs>
            </div>
        </>
    );
}

export default LevelUpContent;
