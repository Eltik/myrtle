import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import { Badge } from "~/components/ui/badge";
import Image from "next/image";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/tooltip";
import type { Operator, OperatorPhase } from "~/types/impl/api/static/operator";
import type { Item } from "~/types/impl/api/static/material";

function LevelUpContent({ operator }: { operator: Operator }) {
    const [materials, setMaterials] = useState<Item[]>([]);
    const [activeTab, setActiveTab] = useState<"elite" | "skill">("elite");
    const [elitePromotionCosts, setElitePromotionCosts] = useState<{
        elite: string;
        materials: {
            quantity: number;
            material: {
                itemId: string;
                name: string;
            };
        }[];
    }[]>([]);

    const [skillLevelUpCosts, setSkillLevelUpCosts] = useState<{
        level: string;
        phase: OperatorPhase,
        materials: {
            quantity: number;
            material: {
                itemId: string;
                name: string;
            };
        }[];
    }[]>([]);

    // Separate useEffect for setting up costs based on operator
    useEffect(() => {
        const elitePromCosts = operator.phases
            .filter((phase, index) => index > 0 && phase.evolveCost) // Skip first phase (E0) as it has no promotion cost
            .map((phase, index) => {
                return {
                    elite: `E${index + 1}`,
                    materials: phase.evolveCost?.map((cost) => ({
                        quantity: cost.count,
                        material: {
                            itemId: cost.id,
                            name: cost.id
                        },
                    })) ?? [],
                };
            });

        setElitePromotionCosts(elitePromCosts);

        const skillLevelCosts = Array.isArray(operator.allSkillLvlup) ? operator.allSkillLvlup.map((levelUp) => {
            return {
                level: `Level ${levelUp.unlockCond.level}`,
                phase: levelUp.unlockCond.phase,
                materials: levelUp.lvlUpCost.map((cost) => ({
                    quantity: cost.count,
                    material: {
                        itemId: cost.id,
                        name: cost.id
                    },
                })) ?? [],
            };
        }) : [];

        setSkillLevelUpCosts(skillLevelCosts);
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
                ).json()) as { data: { items: Record<string, Item>; } };

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
    const renderMaterialItem = (material: { quantity: number; material: { itemId: string; name: string; } }) => {
        const materialData = fetchMaterial(material.material.itemId);
        
        return (
            <TooltipProvider key={material.material.itemId}>
                <Tooltip>
                    <TooltipTrigger asChild>
                        <div className="flex flex-col items-center">
                            <div className="relative">
                                {materialData ? (
                                    <Image
                                        src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/item/${materialData.iconId}.png`}
                                        alt={materialData.name ?? material.material.name}
                                        width={50}
                                        height={50}
                                        className="rounded-md"
                                    />
                                ) : (
                                    <div className="flex h-[50px] w-[50px] items-center justify-center rounded-md bg-gray-200">
                                        <span className="text-xs">Loading...</span>
                                    </div>
                                )}
                                <Badge className="absolute -bottom-2 -right-2 flex h-5 min-w-5 items-center justify-center rounded-full p-0 px-1 text-xs">
                                    {material.quantity}
                                </Badge>
                            </div>
                            <span className="mt-2 text-xs line-clamp-1">{materialData?.name ?? material.material.name}</span>
                        </div>
                    </TooltipTrigger>
                    <TooltipContent>
                        <p>{materialData?.name ?? material.material.name} x{material.quantity}</p>
                    </TooltipContent>
                </Tooltip>
            </TooltipProvider>
        );
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
                                        <CardContent>
                                            {promotion.materials && promotion.materials.length > 0 ? (
                                                <div className="grid grid-cols-3 gap-4 sm:grid-cols-4 md:grid-cols-3 lg:grid-cols-4">
                                                    {promotion.materials.map((material) => renderMaterialItem(material))}
                                                </div>
                                            ) : (
                                                <p className="text-muted-foreground">No materials required.</p>
                                            )}
                                        </CardContent>
                                    </Card>
                                ))}
                            </div>
                        )}
                    </div>
                </TabsContent>
                
                <TabsContent value="skill" className="mt-4">
                    <div className="space-y-4">
                        <h2 className="text-xl font-bold">Skill Level Up Costs</h2>
                        
                        {skillLevelUpCosts.length === 0 ? (
                            <p className="text-muted-foreground">No skill level up costs available for this operator.</p>
                        ) : (
                            <div className="space-y-4">
                                {skillLevelUpCosts.map((levelUp, index) => (
                                    <Card key={index}>
                                        <CardHeader className="pb-2">
                                            <CardTitle className="text-lg">
                                                <Badge variant="outline" className="mr-2">
                                                    {levelUp.phase.replace("PHASE_", "E")}
                                                </Badge>
                                                {levelUp.level}
                                            </CardTitle>
                                        </CardHeader>
                                        <CardContent>
                                            {levelUp.materials && levelUp.materials.length > 0 ? (
                                                <div className="grid grid-cols-3 gap-4 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6">
                                                    {levelUp.materials.map((material) => renderMaterialItem(material))}
                                                </div>
                                            ) : (
                                                <p className="text-muted-foreground">No materials required.</p>
                                            )}
                                        </CardContent>
                                    </Card>
                                ))}
                            </div>
                        )}
                    </div>
                </TabsContent>
            </Tabs>
        </div>
    );
}

export default LevelUpContent;
