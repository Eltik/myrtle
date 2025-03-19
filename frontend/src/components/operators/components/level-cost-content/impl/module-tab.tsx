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
import type { MaterialCost } from "~/types/impl/frontend/impl/operators";
import { getGridColumns, calculateTotalModuleMaterials } from "./helper";
import { MaterialItem } from "./material-item";

interface ModuleTabProps {
    operator: Operator;
    moduleCosts: Record<string, MaterialCost[][]>;
    materials: Item[];
}

export const ModuleTab = ({ operator, moduleCosts, materials }: ModuleTabProps) => {
    const [activeModuleTab, setActiveModuleTab] = useState<string>("");
    const [moduleLevel, setModuleLevel] = useState(0);
    const [showTotalModuleCost, setShowTotalModuleCost] = useState(false);

    // Set default active module tab if operator has modules
    useEffect(() => {
        if (operator.modules?.length > 0) {
            const firstModuleId = operator.modules[0]?.uniEquipId;
            if (firstModuleId) {
                setActiveModuleTab(firstModuleId);
            }
        }
    }, [operator.modules]);

    // Get the current module's costs
    const currentModuleCosts = activeModuleTab ? (moduleCosts[activeModuleTab] ?? []) : [];

    // Get the maximum module level for the slider (0-indexed, so Stage 1-3 = index 0-2)
    const maxModuleLevel = currentModuleCosts.length - 1;

    // Ensure module level is within bounds
    useEffect(() => {
        if (maxModuleLevel >= 0 && moduleLevel > maxModuleLevel) {
            setModuleLevel(maxModuleLevel);
        }
    }, [activeModuleTab, maxModuleLevel, moduleLevel]);

    // Get the current module level costs
    const currentModuleLevelCosts = currentModuleCosts[moduleLevel] ?? [];

    // Get total module materials if checkbox is checked
    const totalModuleMaterials = showTotalModuleCost && activeModuleTab ? calculateTotalModuleMaterials(currentModuleCosts, activeModuleTab, moduleLevel) : null;

    return (
        <div className="space-y-4">
            <h2 className="text-xl font-bold">Module Level Up Costs</h2>

            {!operator.modules || operator.modules.length === 0 ? (
                <p className="text-muted-foreground">No modules available for this operator.</p>
            ) : (
                <div className="space-y-4">
                    {/* Module Tabs */}
                    <Tabs defaultValue={operator.modules[0]?.uniEquipId} value={activeModuleTab} onValueChange={setActiveModuleTab} className="w-full">
                        <TabsList
                            className="grid w-full"
                            style={{
                                gridTemplateColumns: `repeat(${operator.modules.length || 1}, minmax(0, 1fr))`,
                            }}
                        >
                            {operator.modules.map((module) => (
                                <TabsTrigger value={module.uniEquipId} key={module.uniEquipId} className="truncate px-2 text-sm">
                                    <span className="truncate" title={module.uniEquipName}>
                                        {module.uniEquipName}
                                    </span>
                                </TabsTrigger>
                            ))}
                        </TabsList>

                        {/* Module Level Slider */}
                        <div className="mb-6 mt-4 flex flex-col">
                            <div className="flex flex-col">
                                <span className="text-lg font-bold">Module Level</span>
                                <span className="text-sm text-muted-foreground">Drag the slider to view costs for different module levels</span>
                            </div>
                            <div className="flex max-w-[80%] flex-row items-center gap-2">
                                <Slider className="w-full" defaultValue={[0]} value={[moduleLevel]} onValueChange={(value) => setModuleLevel(value[0] ?? 0)} min={0} max={maxModuleLevel >= 0 ? maxModuleLevel : 0} step={1} disabled={maxModuleLevel < 0} />
                                <div className="flex items-center gap-2">
                                    <Badge variant="outline" className="flex h-8 min-w-20 items-center justify-center text-center">
                                        <span>Stage {moduleLevel + 1}</span>
                                    </Badge>
                                </div>
                            </div>

                            {/* Total Cost Checkbox */}
                            <div className="mt-4 flex items-center space-x-2">
                                <Checkbox id="showTotalModuleCost" checked={showTotalModuleCost} onCheckedChange={(checked) => setShowTotalModuleCost(checked === true)} disabled={maxModuleLevel < 0} />
                                <Label htmlFor="showTotalModuleCost" className={maxModuleLevel < 0 ? "text-muted-foreground" : ""}>
                                    Show total cost from Stage 1 to Stage {moduleLevel + 1}
                                </Label>
                            </div>
                        </div>

                        {/* Module Cost Content */}
                        {operator.modules.map((module) => (
                            <TabsContent value={module.uniEquipId} key={module.uniEquipId}>
                                <Card>
                                    <CardHeader>
                                        <CardTitle className="flex items-center gap-2">
                                            {showTotalModuleCost ? "Total Cost" : "Level Up Cost"}
                                            <Image
                                                src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/equip/type/${module.typeIcon}.png`}
                                                width={24}
                                                height={24}
                                                alt={module.typeName1}
                                                style={{
                                                    maxWidth: "100%",
                                                    height: "auto",
                                                    objectFit: "contain",
                                                }}
                                            />
                                            <span className="text-sm">
                                                {module.typeName1} {module.typeName2 ? `- ${module.typeName2}` : ""}
                                            </span>
                                        </CardTitle>
                                    </CardHeader>
                                    <CardContent>
                                        {!currentModuleCosts || currentModuleCosts.length === 0 ? (
                                            <p className="text-muted-foreground">No level-up costs available for this module.</p>
                                        ) : showTotalModuleCost ? (
                                            totalModuleMaterials && totalModuleMaterials.length > 0 ? (
                                                <div className={`grid gap-4 ${getGridColumns(totalModuleMaterials.length)}`}>
                                                    {totalModuleMaterials.map((material) => (
                                                        <MaterialItem key={material.material.itemId} material={material} materials={materials} />
                                                    ))}
                                                </div>
                                            ) : (
                                                <p className="text-muted-foreground">No materials required for module upgrades.</p>
                                            )
                                        ) : currentModuleLevelCosts && currentModuleLevelCosts.length > 0 ? (
                                            <div className={`grid gap-4 ${getGridColumns(currentModuleLevelCosts.length)}`}>
                                                {currentModuleLevelCosts.map((material) => (
                                                    <MaterialItem key={material.material.itemId} material={material} materials={materials} />
                                                ))}
                                            </div>
                                        ) : (
                                            <p className="text-muted-foreground">No materials required for Stage {moduleLevel + 1}.</p>
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
