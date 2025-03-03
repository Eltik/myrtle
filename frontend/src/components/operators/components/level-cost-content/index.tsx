import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import { Badge } from "~/components/ui/badge";
import Image from "next/image";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/tooltip";
import type { Operator } from "~/types/impl/api/static/operator";
import type { Item } from "~/types/impl/api/static/material";
import { Slider } from "~/components/ui/slider";
import { Checkbox } from "~/components/ui/checkbox";
import { Label } from "~/components/ui/label";
import type { MaterialCost, SkillLevelCost } from "~/types/impl/frontend/impl/operators";

function LevelUpContent({ operator }: { operator: Operator }) {
    const [materials, setMaterials] = useState<Item[]>([]);
    const [activeTab, setActiveTab] = useState<"elite" | "skill">("elite");
    const [elitePromotionCosts, setElitePromotionCosts] = useState<
        {
            elite: string;
            materials: MaterialCost[];
        }[]
    >([]);

    // New state for skill slider and total cost checkbox
    const [skillLevel, setSkillLevel] = useState(0);
    const [showTotalCost, setShowTotalCost] = useState(false);
    const [activeSkillTab, setActiveSkillTab] = useState<string>("");

    // Store skill level costs per skill
    const [skillLevelCosts, setSkillLevelCosts] = useState<Record<string, SkillLevelCost[]>>({});

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

        // Set default active skill tab if operator has skills
        if (operator.skills?.length > 0) {
            const firstSkillId = operator.skills[0]?.skillId;
            if (firstSkillId) {
                setActiveSkillTab(firstSkillId);
            }
        }
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

    function fetchMaterial(itemId: string): Item | null {
        if (materials.length > 0) {
            return materials.find((item) => item.itemId === itemId) ?? null;
        } else {
            return null;
        }
    }

    // Helper function to render material items with quantity
    const renderMaterialItem = (material: MaterialCost) => {
        const materialData = fetchMaterial(material.material.itemId);

        return (
            <TooltipProvider key={material.material.itemId}>
                <Tooltip>
                    <TooltipTrigger asChild>
                        <div className="flex flex-col items-center">
                            <div className="relative">
                                {materialData ? (
                                    <Image src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/item/${materialData.iconId}.png`} alt={materialData.name ?? material.material.name} width={50} height={50} className="rounded-md" />
                                ) : (
                                    <div className="flex h-[50px] w-[50px] items-center justify-center rounded-md bg-gray-200">
                                        <span className="text-xs">Loading...</span>
                                    </div>
                                )}
                                <Badge className="absolute -bottom-2 -right-2 flex h-5 min-w-5 items-center justify-center rounded-full p-0 px-1 text-xs">{material.quantity}</Badge>
                            </div>
                            <span className="mt-2 line-clamp-1 text-xs">{materialData?.name ?? material.material.name}</span>
                        </div>
                    </TooltipTrigger>
                    <TooltipContent>
                        <p>
                            {materialData?.name ?? material.material.name} x{material.quantity}
                        </p>
                    </TooltipContent>
                </Tooltip>
            </TooltipProvider>
        );
    };

    // Helper function to calculate total materials needed up to a specific level for a specific skill
    const calculateTotalMaterials = (skillId: string, upToIndex: number) => {
        const costs = skillLevelCosts[skillId];
        if (!costs || upToIndex < 0 || costs.length === 0) {
            return [];
        }

        const totalMaterials: Record<string, MaterialCost> = {};

        // Combine materials from level 1 up to the selected level
        for (let i = 0; i <= upToIndex && i < costs.length; i++) {
            const levelCost = costs[i];
            if (levelCost?.materials) {
                levelCost.materials.forEach((material) => {
                    const itemId = material.material.itemId;
                    if (totalMaterials[itemId]) {
                        totalMaterials[itemId].quantity += material.quantity;
                    } else {
                        totalMaterials[itemId] = { ...material };
                    }
                });
            }
        }

        return Object.values(totalMaterials);
    };

    // Get the current skill's costs
    const currentSkillCosts = activeSkillTab ? (skillLevelCosts[activeSkillTab] ?? []) : [];

    // Get the maximum skill level for the slider
    const maxSkillLevel = currentSkillCosts.length - 1;

    // Ensure skill level is within bounds for the current skill
    useEffect(() => {
        if (maxSkillLevel >= 0 && skillLevel > maxSkillLevel) {
            setSkillLevel(maxSkillLevel);
        }
    }, [activeSkillTab, maxSkillLevel, skillLevel]);

    // Get the current level cost to display
    const currentLevelCost = currentSkillCosts[skillLevel];

    // Get total materials if checkbox is checked
    const totalMaterials = showTotalCost && activeSkillTab ? calculateTotalMaterials(activeSkillTab, skillLevel) : null;

    // Helper function to get the grid columns based on the number of materials
    const getGridColumns = (materialCount: number) => {
        if (materialCount <= 3) return "grid-cols-3";
        if (materialCount <= 4) return "grid-cols-4";
        if (materialCount <= 6) return "grid-cols-6";
        return "grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6";
    };

    return (
        <div className="p-4">
            <Tabs value={activeTab} onValueChange={(value) => setActiveTab(value as "elite" | "skill")}>
                <TabsList className="grid w-full grid-cols-2">
                    <TabsTrigger value="elite">Elite Promotion</TabsTrigger>
                    <TabsTrigger value="skill">Skill Level Up</TabsTrigger>
                </TabsList>

                <TabsContent value="elite" className="mt-4">
                    <div className="space-y-4">
                        <h2 className="text-xl font-bold">Elite Promotion Costs</h2>

                        {elitePromotionCosts.length === 0 ? (
                            <p className="text-muted-foreground">No promotion costs available for this operator.</p>
                        ) : (
                            <div className="grid gap-4 md:grid-cols-2">
                                {elitePromotionCosts.map((promotion) => (
                                    <Card key={promotion.elite}>
                                        <CardHeader className="pb-2">
                                            <CardTitle className="text-lg">
                                                <Badge variant="outline" className="mr-2">
                                                    {promotion.elite}
                                                </Badge>
                                                Promotion
                                            </CardTitle>
                                        </CardHeader>
                                        <CardContent>{promotion.materials && promotion.materials.length > 0 ? <div className="grid grid-cols-3 gap-4 sm:grid-cols-4 md:grid-cols-3 lg:grid-cols-4">{promotion.materials.map((material) => renderMaterialItem(material))}</div> : <p className="text-muted-foreground">No materials required.</p>}</CardContent>
                                    </Card>
                                ))}
                            </div>
                        )}
                    </div>
                </TabsContent>

                <TabsContent value="skill" className="mt-4">
                    <div className="space-y-4">
                        <h2 className="text-xl font-bold">Skill Level Up Costs</h2>

                        {!operator.skills || operator.skills.length === 0 ? (
                            <p className="text-muted-foreground">No skills available for this operator.</p>
                        ) : (
                            <div className="space-y-4">
                                {/* Skill Tabs */}
                                <Tabs defaultValue={operator.skills[0]?.skillId} value={activeSkillTab} onValueChange={setActiveSkillTab} className="w-full">
                                    <TabsList className="grid w-full" style={{ gridTemplateColumns: `repeat(${operator.skills.length || 1}, minmax(0, 1fr))` }}>
                                        {operator.skills.map((skill) => (
                                            <TabsTrigger value={skill.skillId} key={skill.skillId} className="truncate px-2 text-sm">
                                                <span className="truncate" title={skill.static?.levels[0]?.name}>
                                                    {skill.static?.levels[0]?.name}
                                                </span>
                                            </TabsTrigger>
                                        ))}
                                    </TabsList>

                                    {/* Skill Level Slider */}
                                    <div className="mb-6 mt-4 flex flex-col">
                                        <div className="flex flex-col">
                                            <span className="text-lg font-bold">Skill Level</span>
                                            <span className="text-sm text-muted-foreground">Drag the slider to view costs for different skill levels</span>
                                        </div>
                                        <div className="flex max-w-[80%] flex-col items-center gap-2 md:flex-row">
                                            <Slider className="w-full" defaultValue={[0]} value={[skillLevel]} onValueChange={(value) => setSkillLevel(value[0] ?? 0)} min={0} max={maxSkillLevel >= 0 ? maxSkillLevel : 0} step={1} disabled={maxSkillLevel < 0} />
                                            <div className="flex items-center gap-2">
                                                <Badge variant="outline" className="flex h-8 min-w-16 items-center justify-center text-center">
                                                    {currentLevelCost && currentLevelCost?.level <= 7 ? (
                                                        <span>Level {currentLevelCost?.level}</span>
                                                    ) : (
                                                        <Image
                                                            src={`/m-${currentLevelCost?.level === 8 ? "1" : currentLevelCost?.level === 9 ? "2" : "3"}_0.webp`}
                                                            className="h-8 w-9"
                                                            width={40}
                                                            height={40}
                                                            alt="M1"
                                                            style={{
                                                                maxWidth: "100%",
                                                                height: "auto",
                                                            }}
                                                        />
                                                    )}
                                                </Badge>
                                                {currentLevelCost?.phase && (
                                                    <Badge variant="outline" className="flex min-w-12 items-center justify-center text-center">
                                                        <Image
                                                            src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${currentLevelCost.phase.replace("PHASE_", "")}.png`}
                                                            width={35}
                                                            height={35}
                                                            alt="Promotion"
                                                            style={{
                                                                maxWidth: "100%",
                                                                height: "auto",
                                                                objectFit: "contain",
                                                            }}
                                                        />
                                                    </Badge>
                                                )}
                                            </div>
                                        </div>

                                        {/* Total Cost Checkbox */}
                                        <div className="mt-4 flex items-center space-x-2">
                                            <Checkbox id="showTotalCost" checked={showTotalCost} onCheckedChange={(checked) => setShowTotalCost(checked === true)} disabled={maxSkillLevel < 0} />
                                            <Label htmlFor="showTotalCost" className={maxSkillLevel < 0 ? "text-muted-foreground" : ""}>
                                                Show total cost from Level 2 to {currentLevelCost && currentLevelCost?.level > 7 ? `M${currentLevelCost.level - 7}` : `Level ${currentLevelCost?.level}`}
                                            </Label>
                                        </div>
                                    </div>

                                    {/* Skill Cost Content */}
                                    {operator.skills.map((skill) => (
                                        <TabsContent value={skill.skillId} key={skill.skillId}>
                                            <Card>
                                                <CardHeader>
                                                    <CardTitle>{showTotalCost ? "Total Cost" : "Level Up Cost"}</CardTitle>
                                                </CardHeader>
                                                <CardContent>{currentSkillCosts.length === 0 ? <p className="text-muted-foreground">No level-up costs available for this skill.</p> : showTotalCost ? totalMaterials && totalMaterials.length > 0 ? <div className={`grid gap-4 ${getGridColumns(totalMaterials.length)}`}>{totalMaterials.map((material) => renderMaterialItem(material))}</div> : <p className="text-muted-foreground">No materials required.</p> : currentLevelCost?.materials && currentLevelCost.materials.length > 0 ? <div className={`grid gap-4 ${getGridColumns(currentLevelCost.materials.length)}`}>{currentLevelCost.materials.map((material) => renderMaterialItem(material))}</div> : <p className="text-muted-foreground">No materials required for this level.</p>}</CardContent>
                                            </Card>
                                        </TabsContent>
                                    ))}
                                </Tabs>
                            </div>
                        )}
                    </div>
                </TabsContent>
            </Tabs>
        </div>
    );
}

export default LevelUpContent;
