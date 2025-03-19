import { useState, useEffect } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import { Badge } from "~/components/ui/badge";
import Image from "next/image";
import { Slider } from "~/components/ui/slider";
import { Checkbox } from "~/components/ui/checkbox";
import { Label } from "~/components/ui/label";
import type { Item } from "~/types/impl/api/static/material";
import type { Operator } from "~/types/impl/api/static/operator";
import type { SkillLevelCost } from "~/types/impl/frontend/impl/operators";
import { getGridColumns, calculateTotalMaterials } from "./helper";
import { MaterialItem } from "./material-item";

interface SkillLevelUpTabProps {
    operator: Operator;
    skillLevelCosts: Record<string, SkillLevelCost[]>;
    materials: Item[];
}

export const SkillLevelUpTab = ({ operator, skillLevelCosts, materials }: SkillLevelUpTabProps) => {
    const [activeSkillTab, setActiveSkillTab] = useState<string>("");
    const [skillLevel, setSkillLevel] = useState(0);
    const [showTotalCost, setShowTotalCost] = useState(false);

    // Set default active skill tab if operator has skills
    useEffect(() => {
        if (operator.skills?.length > 0) {
            const firstSkillId = operator.skills[0]?.skillId;
            if (firstSkillId) {
                setActiveSkillTab(firstSkillId);
            }
        }
    }, [operator.skills]);

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
    const totalMaterials = showTotalCost && activeSkillTab ? calculateTotalMaterials(currentSkillCosts, activeSkillTab, skillLevel) : null;

    return (
        <div className="space-y-4">
            <h2 className="text-xl font-bold">Skill Level Up Costs</h2>

            {!operator.skills || operator.skills.length === 0 ? (
                <p className="text-muted-foreground">No skills available for this operator.</p>
            ) : (
                <div className="space-y-4">
                    {/* Skill Tabs */}
                    <Tabs defaultValue={operator.skills[0]?.skillId} value={activeSkillTab} onValueChange={setActiveSkillTab} className="w-full">
                        <TabsList
                            className="grid w-full"
                            style={{
                                gridTemplateColumns: `repeat(${operator.skills.length || 1}, minmax(0, 1fr))`,
                            }}
                        >
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
                                    <CardContent>
                                        {currentSkillCosts.length === 0 ? (
                                            <p className="text-muted-foreground">No level-up costs available for this skill.</p>
                                        ) : showTotalCost ? (
                                            totalMaterials && totalMaterials.length > 0 ? (
                                                <div className={`grid gap-4 ${getGridColumns(totalMaterials.length)}`}>
                                                    {totalMaterials.map((material) => (
                                                        <MaterialItem key={material.material.itemId} material={material} materials={materials} />
                                                    ))}
                                                </div>
                                            ) : (
                                                <p className="text-muted-foreground">No materials required.</p>
                                            )
                                        ) : currentLevelCost?.materials && currentLevelCost.materials.length > 0 ? (
                                            <div className={`grid gap-4 ${getGridColumns(currentLevelCost.materials.length)}`}>
                                                {currentLevelCost.materials.map((material) => (
                                                    <MaterialItem key={material.material.itemId} material={material} materials={materials} />
                                                ))}
                                            </div>
                                        ) : (
                                            <p className="text-muted-foreground">No materials required for this level.</p>
                                        )}
                                    </CardContent>
                                </Card>
                            </TabsContent>
                        ))}
                    </Tabs>
                </div>
            )}
        </div>
    );
};
