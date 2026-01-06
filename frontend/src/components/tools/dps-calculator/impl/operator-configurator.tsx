"use client";

import { ChevronDown, Trash2 } from "lucide-react";
import { useTheme } from "next-themes";
import { useState } from "react";
import { RARITY_COLORS, RARITY_COLORS_LIGHT } from "~/components/operators/list/constants";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "~/components/ui/motion-primitives/disclosure";
import { Button } from "~/components/ui/shadcn/button";
import { Label } from "~/components/ui/shadcn/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Slider } from "~/components/ui/shadcn/slider";
import type { OperatorConfiguration } from "./types";

interface OperatorConfiguratorProps {
    operator: OperatorConfiguration;
    onUpdate: (id: string, updates: Partial<OperatorConfiguration>) => void;
    onRemove: (id: string) => void;
}

export function OperatorConfigurator({ operator, onUpdate, onRemove }: OperatorConfiguratorProps) {
    const [isOpen, setIsOpen] = useState(true);
    const { resolvedTheme } = useTheme();

    const rarityColors = resolvedTheme === "light" ? RARITY_COLORS_LIGHT : RARITY_COLORS;
    const rarityColor = rarityColors[operator.rarity] ?? "#ffffff";

    const updateParams = (key: keyof typeof operator.params, value: number) => {
        onUpdate(operator.id, {
            params: { ...operator.params, [key]: value },
        });
    };

    return (
        <div className="overflow-hidden rounded-lg border border-border bg-card">
            <Disclosure onOpenChange={setIsOpen} open={isOpen}>
                <DisclosureTrigger>
                    <div className="flex w-full items-center justify-between p-3 text-left hover:bg-muted/30 data-[state=open]:bg-muted/30">
                        <div className="flex items-center gap-3">
                            <div className="h-8 w-1 shrink-0 rounded-full" style={{ backgroundColor: operator.color }} />
                            <div>
                                <div className="font-semibold text-sm">{operator.operatorName}</div>
                                <div className="flex items-center gap-1.5 text-muted-foreground text-xs">
                                    <span style={{ color: rarityColor }}>{"★".repeat(operator.rarity)}</span>
                                    {operator.availableSkills.length > 0 && (
                                        <>
                                            <span>•</span>
                                            <span>
                                                S{operator.params.skillIndex || operator.availableSkills[0] || 1}
                                                {operator.maxPromotion >= 2 && `M${operator.params.masteryLevel || 0}`}
                                            </span>
                                        </>
                                    )}
                                </div>
                            </div>
                        </div>
                        <div className="flex items-center gap-2">
                            <Button
                                className="h-8 w-8"
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onRemove(operator.id);
                                }}
                                size="icon"
                                variant="ghost"
                            >
                                <Trash2 className="h-4 w-4" />
                            </Button>
                            <ChevronDown className="h-4 w-4 text-muted-foreground transition-transform duration-200 group-data-[state=open]:rotate-180" />
                        </div>
                    </div>
                </DisclosureTrigger>

                <DisclosureContent>
                    <div className="border-border/50 border-t p-4">
                        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
                            {/* Skill Selection - only show if operator has skills */}
                            {operator.availableSkills.length > 0 && (
                                <div className="space-y-2">
                                    <Label className="text-xs">Skill</Label>
                                    <Select onValueChange={(value) => updateParams("skillIndex", Number(value))} value={String(operator.params.skillIndex || operator.availableSkills[0] || 1)}>
                                        <SelectTrigger>
                                            <SelectValue />
                                        </SelectTrigger>
                                        <SelectContent>
                                            {operator.availableSkills.map((skill) => (
                                                <SelectItem key={skill} value={String(skill)}>
                                                    Skill {skill}
                                                </SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                </div>
                            )}

                            {/* Mastery Level - only show if operator has skills and is E2 capable */}
                            {operator.availableSkills.length > 0 && operator.maxPromotion >= 2 && (
                                <div className="space-y-2">
                                    <Label className="text-xs">Mastery Level: {operator.params.masteryLevel || 0}</Label>
                                    <Slider className="pt-2" max={3} min={0} onValueChange={(values) => updateParams("masteryLevel", values[0] ?? 0)} step={1} value={[operator.params.masteryLevel || 0]} />
                                </div>
                            )}

                            {/* Module - only show if operator has modules */}
                            {operator.availableModules.length > 0 && (
                                <div className="space-y-2">
                                    <Label className="text-xs">Module</Label>
                                    <Select onValueChange={(value) => updateParams("moduleIndex", Number(value))} value={String(operator.params.moduleIndex || 0)}>
                                        <SelectTrigger>
                                            <SelectValue />
                                        </SelectTrigger>
                                        <SelectContent>
                                            <SelectItem value="0">None</SelectItem>
                                            {operator.availableModules.map((moduleIdx) => (
                                                <SelectItem key={moduleIdx} value={String(moduleIdx)}>
                                                    Module {moduleIdx === 1 ? "X" : moduleIdx === 2 ? "Y" : "Δ"}
                                                </SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                </div>
                            )}

                            {/* Potential */}
                            <div className="space-y-2">
                                <Label className="text-xs">Potential: {operator.params.potential || 1}</Label>
                                <Slider className="pt-2" max={6} min={1} onValueChange={(values) => updateParams("potential", values[0] ?? 1)} step={1} value={[operator.params.potential || 1]} />
                            </div>

                            {/* Trust */}
                            <div className="space-y-2">
                                <Label className="text-xs">Trust: {operator.params.trust || 0}%</Label>
                                <Slider className="pt-2" max={100} min={0} onValueChange={(values) => updateParams("trust", values[0] ?? 0)} step={10} value={[operator.params.trust || 0]} />
                            </div>

                            {/* Elite/Level - only show options up to maxPromotion */}
                            <div className="space-y-2">
                                <Label className="text-xs">Elite Level</Label>
                                <Select onValueChange={(value) => updateParams("promotion", Number(value))} value={String(operator.params.promotion ?? operator.maxPromotion)}>
                                    <SelectTrigger>
                                        <SelectValue />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {Array.from({ length: operator.maxPromotion + 1 }, (_, i) => (
                                            // biome-ignore lint/suspicious/noArrayIndexKey: Static array of promotion phases
                                            <SelectItem key={`elite-${i}`} value={String(i)}>
                                                Elite {i}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                        </div>
                    </div>
                </DisclosureContent>
            </Disclosure>
        </div>
    );
}
