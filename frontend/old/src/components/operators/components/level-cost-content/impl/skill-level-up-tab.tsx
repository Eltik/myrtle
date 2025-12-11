import Image from "next/image";
import { useEffect, useState } from "react";
import { Badge } from "~/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Checkbox } from "~/components/ui/checkbox";
import { Label } from "~/components/ui/label";
import { Slider } from "~/components/ui/slider";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import type { Item } from "~/types/impl/api/static/material";
import type { Operator } from "~/types/impl/api/static/operator";
import type { SkillLevelCost } from "~/types/impl/frontend/impl/operators";
import { calculateTotalMaterials, getGridColumns } from "./helper";
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
    }, [maxSkillLevel, skillLevel]);

    // Get the current level cost to display
    const currentLevelCost = currentSkillCosts[skillLevel];

    // Get total materials if checkbox is checked
    const totalMaterials = showTotalCost && activeSkillTab ? calculateTotalMaterials(currentSkillCosts, activeSkillTab, skillLevel) : null;

    return (
        <div className="space-y-4">
            <h2 className="font-bold text-xl">Skill Level Up Costs</h2>

            {!operator.skills || operator.skills.length === 0 ? (
                <p className="text-muted-foreground">No skills available for this operator.</p>
            ) : (
                <div className="space-y-4">
                    {/* Skill Tabs */}
                    <Tabs className="w-full" defaultValue={operator.skills[0]?.skillId} onValueChange={setActiveSkillTab} value={activeSkillTab}>
                        <TabsList
                            className="grid w-full"
                            style={{
                                gridTemplateColumns: `repeat(${operator.skills.length || 1}, minmax(0, 1fr))`,
                            }}
                        >
                            {operator.skills.map((skill) => (
                                <TabsTrigger className="truncate px-2 text-sm" key={skill.skillId} value={skill.skillId}>
                                    <span className="truncate" title={skill.static?.levels[0]?.name}>
                                        {skill.static?.levels[0]?.name}
                                    </span>
                                </TabsTrigger>
                            ))}
                        </TabsList>

                        {/* Skill Level Slider */}
                        <div className="mt-4 mb-6 flex flex-col">
                            <div className="flex flex-col">
                                <span className="font-bold text-lg">Skill Level</span>
                                <span className="text-muted-foreground text-sm">Drag the slider to view costs for different skill levels</span>
                            </div>
                            <div className="flex max-w-[80%] flex-col items-center gap-2 md:flex-row">
                                <Slider className="w-full" defaultValue={[0]} disabled={maxSkillLevel < 0} max={maxSkillLevel >= 0 ? maxSkillLevel : 0} min={0} onValueChange={(value) => setSkillLevel(value[0] ?? 0)} step={1} value={[skillLevel]} />
                                <div className="flex items-center gap-2">
                                    <Badge className="flex h-8 min-w-16 items-center justify-center text-center" variant="outline">
                                        {currentLevelCost && currentLevelCost?.level <= 7 ? (
                                            <span>Level {currentLevelCost?.level}</span>
                                        ) : (
                                            <Image
                                                alt="M1"
                                                className="h-8 w-9"
                                                height={40}
                                                src={`/m-${currentLevelCost?.level === 8 ? "1" : currentLevelCost?.level === 9 ? "2" : "3"}_0.webp`}
                                                style={{
                                                    maxWidth: "100%",
                                                    height: "auto",
                                                }}
                                                width={40}
                                            />
                                        )}
                                    </Badge>
                                    {currentLevelCost?.phase && (
                                        <Badge className="flex min-w-12 items-center justify-center text-center" variant="outline">
                                            <Image
                                                alt="Promotion"
                                                height={35}
                                                src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${currentLevelCost.phase.replace("PHASE_", "")}.png`}
                                                style={{
                                                    maxWidth: "100%",
                                                    height: "auto",
                                                    objectFit: "contain",
                                                }}
                                                width={35}
                                            />
                                        </Badge>
                                    )}
                                </div>
                            </div>

                            {/* Total Cost Checkbox */}
                            <div className="mt-4 flex items-center space-x-2">
                                <Checkbox checked={showTotalCost} disabled={maxSkillLevel < 0} id="showTotalCost" onCheckedChange={(checked) => setShowTotalCost(checked === true)} />
                                <Label className={maxSkillLevel < 0 ? "text-muted-foreground" : ""} htmlFor="showTotalCost">
                                    Show total cost from Level 2 to {currentLevelCost && currentLevelCost?.level > 7 ? `M${currentLevelCost.level - 7}` : `Level ${currentLevelCost?.level}`}
                                </Label>
                            </div>
                        </div>

                        {/* Skill Cost Content */}
                        {operator.skills.map((skill) => (
                            <TabsContent key={skill.skillId} value={skill.skillId}>
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
