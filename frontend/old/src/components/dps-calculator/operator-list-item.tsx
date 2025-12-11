import { AnimatePresence, motion } from "framer-motion";
import { ChevronDown, Copy, X } from "lucide-react";
import Image from "next/image";
import { useEffect, useRef, useState } from "react";
import type { OperatorParams } from "~/types/impl/api/impl/dps-calculator";
import { OperatorRarity } from "~/types/impl/api/static/operator";
import type { OperatorListItemProps } from "~/types/impl/frontend/impl/dps-calculator";
import { Button } from "../ui/button";
import { Card, CardContent } from "../ui/card";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";
import { Switch } from "../ui/switch";

export default function OperatorListItem({ operator, onParamsChange, onRemove, onDuplicate }: OperatorListItemProps) {
    const [params, setParams] = useState<OperatorParams>({});
    const [isOpen, setIsOpen] = useState(false);
    const initializedRef = useRef(false);

    const handleParamChange = (key: keyof OperatorParams, value: OperatorParams[keyof OperatorParams]) => {
        const newParams = { ...params, [key]: value };
        setParams(newParams);
        onParamsChange(newParams);
    };

    useEffect(() => {
        if (!initializedRef.current) {
            const defaultParams: OperatorParams = {
                skillIndex: operator.defaultSkillIndex,
                masteryLevel: operator.operatorData.data.rarity === OperatorRarity.threeStar ? 6 : 9,
                moduleIndex: operator.defaultModuleIndex,
                moduleLevel: 3,
                buffs: {},
                conditionals: {},
            };

            setParams(defaultParams);
            onParamsChange(defaultParams);
            initializedRef.current = true;
        }
    }, [operator, onParamsChange]);

    return (
        <Card className={`mb-4 w-full ${isOpen ? "" : "transition-all duration-150 hover:bg-primary-foreground"}`}>
            <div>
                <div className="flex items-center justify-between p-4">
                    <button aria-expanded={isOpen} className="flex flex-grow items-center text-left focus:outline-none" onClick={() => setIsOpen(!isOpen)}>
                        <div className="flex items-center">
                            <Image alt={operator.operatorData.data.name} className="mr-4 rounded-full" height={48} src={`https://raw.githubusercontent.com/ArknightsAssets/ArknightsAssets/cn/assets/torappu/dynamicassets/arts/charavatars/${operator.operatorData.data.id}.png`} width={48} />
                            <h3 className="font-semibold text-lg">{operator.displayName}</h3>
                        </div>
                        <motion.div animate={{ rotate: isOpen ? 180 : 0 }} transition={{ duration: 0.3 }}>
                            <ChevronDown className="h-4 w-4" />
                        </motion.div>
                    </button>
                    <div className="flex items-center">
                        <Button
                            aria-label={`Duplicate ${operator.displayName}`}
                            className="ml-1 h-8 w-8 rounded-full"
                            onClick={(e) => {
                                e.stopPropagation();
                                if (operator.instanceId) {
                                    onDuplicate?.(operator.instanceId);
                                }
                            }}
                            size="icon"
                            variant="ghost"
                        >
                            <Copy className="h-4 w-4" />
                        </Button>
                        <Button
                            aria-label={`Remove ${operator.displayName}`}
                            className="ml-1 h-8 w-8 rounded-full"
                            onClick={(e) => {
                                e.stopPropagation();
                                if (operator.instanceId) {
                                    onRemove?.(operator.instanceId);
                                }
                            }}
                            size="icon"
                            variant="ghost"
                        >
                            <X className="h-4 w-4" />
                        </Button>
                    </div>
                </div>
                <AnimatePresence initial={false}>
                    {isOpen && (
                        <motion.div
                            animate="open"
                            exit="collapsed"
                            initial="collapsed"
                            transition={{ duration: 0.3, ease: "easeInOut" }}
                            variants={{
                                open: { opacity: 1, height: "auto" },
                                collapsed: { opacity: 0, height: 0 },
                            }}
                        >
                            <CardContent className="p-4 pt-0">
                                <div className="grid grid-cols-2 gap-4">
                                    {operator.operatorData.data.rarity !== OperatorRarity.twoStar && operator.operatorData.data.rarity !== OperatorRarity.oneStar ? (
                                        <>
                                            <Select onValueChange={(value) => handleParamChange("skillIndex", parseInt(value, 10))} value={String(params.skillIndex)}>
                                                <SelectTrigger>
                                                    <SelectValue placeholder="Select skill" />
                                                </SelectTrigger>
                                                <SelectContent>
                                                    {operator.operatorData.data.skills.map((skill, index) => (
                                                        <SelectItem className="truncate" key={index} value={index.toString()}>
                                                            S{index + 1} - {skill.static?.levels[0]?.name}
                                                        </SelectItem>
                                                    ))}
                                                </SelectContent>
                                            </Select>
                                            <Select onValueChange={(value) => handleParamChange("masteryLevel", parseInt(value, 10))} value={String(params.masteryLevel)}>
                                                <SelectTrigger>
                                                    <SelectValue placeholder="Skill level" />
                                                </SelectTrigger>
                                                <SelectContent>
                                                    {operator.operatorData.data.rarity === OperatorRarity.threeStar
                                                        ? [1, 2, 3, 4, 5, 6, 7]
                                                        : [1, 2, 3, 4, 5, 6, 7, "M1", "M2", "M3"].map((level, index) => (
                                                              <SelectItem key={index} value={index.toString()}>
                                                                  {level}
                                                              </SelectItem>
                                                          ))}
                                                </SelectContent>
                                            </Select>
                                            {operator.operatorData.data.rarity !== OperatorRarity.threeStar ? (
                                                <>
                                                    <Select onValueChange={(value) => handleParamChange("moduleIndex", parseInt(value, 10))} value={String(params.moduleIndex)}>
                                                        <SelectTrigger>
                                                            <SelectValue placeholder="Select module" />
                                                        </SelectTrigger>
                                                        <SelectContent>
                                                            {operator.operatorData.data.modules.map((module, index) => (
                                                                <SelectItem key={index} value={index.toString()}>
                                                                    {module.typeName1 && module.typeName2 ? `${module.typeName1}-${module.typeName2}` : module.uniEquipName}
                                                                </SelectItem>
                                                            ))}
                                                        </SelectContent>
                                                    </Select>
                                                    <Select onValueChange={(value) => handleParamChange("moduleLevel", parseInt(value, 10))} value={String(params.moduleLevel)}>
                                                        <SelectTrigger>
                                                            <SelectValue placeholder="Module level" />
                                                        </SelectTrigger>
                                                        <SelectContent>
                                                            {[1, 2, 3].map((level) => (
                                                                <SelectItem key={level} value={level.toString()}>
                                                                    {level}
                                                                </SelectItem>
                                                            ))}
                                                        </SelectContent>
                                                    </Select>
                                                </>
                                            ) : null}
                                        </>
                                    ) : null}
                                </div>
                                <div className="mt-4">
                                    <h4 className="mb-2 font-semibold text-sm">Buffs</h4>
                                    <div className="grid grid-cols-2 gap-4">
                                        <Input className="input input-bordered w-full" onChange={(e) => handleParamChange("buffs", { ...params.buffs, atk: parseFloat(e.target.value) })} placeholder="ATK % buff" type="number" />
                                        <Input className="input input-bordered w-full" onChange={(e) => handleParamChange("buffs", { ...params.buffs, atkFlat: parseInt(e.target.value, 10) })} placeholder="Flat ATK buff" type="number" />
                                        <Input className="input input-bordered w-full" onChange={(e) => handleParamChange("buffs", { ...params.buffs, aspd: parseInt(e.target.value, 10) })} placeholder="ASPD buff" type="number" />
                                        <Input className="input input-bordered w-full" onChange={(e) => handleParamChange("buffs", { ...params.buffs, fragile: parseFloat(e.target.value) })} placeholder="Fragile debuff" type="number" />
                                    </div>
                                </div>
                                <div className="mt-4">
                                    <h4 className="mb-2 font-semibold text-sm">Conditionals</h4>
                                    <div className="space-y-2">
                                        {operator.talentDamageNames.map((talent) => (
                                            <div className="flex items-center" key={talent}>
                                                <Switch defaultChecked={operator.talentDamage} id={`${operator.instanceId}-${talent}`} onCheckedChange={(checked) => handleParamChange("conditionals", { ...params.conditionals, talentDamage: checked })} />
                                                <Label className="ml-2" htmlFor={`${operator.instanceId}-${talent}`}>
                                                    {talent.charAt(0).toUpperCase() + talent.slice(1)}
                                                </Label>
                                            </div>
                                        ))}
                                        {operator.talent2DamageNames.map((talent) => (
                                            <div className="flex items-center" key={talent}>
                                                <Switch defaultChecked={operator.talent2Damage} id={`${operator.instanceId}-${talent}`} onCheckedChange={(checked) => handleParamChange("conditionals", { ...params.conditionals, talent2Damage: checked })} />
                                                <Label className="ml-2" htmlFor={`${operator.instanceId}-${talent}`}>
                                                    {talent.charAt(0).toUpperCase() + talent.slice(1)}
                                                </Label>
                                            </div>
                                        ))}
                                        {operator.traitDamageNames.map((trait) => (
                                            <div className="flex items-center" key={trait}>
                                                <Switch defaultChecked={operator.traitDamage} id={`${operator.instanceId}-${trait}`} onCheckedChange={(checked) => handleParamChange("conditionals", { ...params.conditionals, traitDamage: checked })} />
                                                <Label className="ml-2" htmlFor={`${operator.instanceId}-${trait}`}>
                                                    {trait.charAt(0).toUpperCase() + trait.slice(1)}
                                                </Label>
                                            </div>
                                        ))}
                                        {operator.skillDamageNames.map((skill) => (
                                            <div className="flex items-center" key={skill}>
                                                <Switch defaultChecked={operator.skillDamage} id={`${operator.instanceId}-${skill}`} onCheckedChange={(checked) => handleParamChange("conditionals", { ...params.conditionals, skillDamage: checked })} />
                                                <Label className="ml-2" htmlFor={`${operator.instanceId}-${skill}`}>
                                                    {skill.charAt(0).toUpperCase() + skill.slice(1)}
                                                </Label>
                                            </div>
                                        ))}
                                        {operator.moduleDamageNames.map((operatorModule) => (
                                            <div className="flex items-center" key={operatorModule}>
                                                <Switch defaultChecked={operator.moduleDamage} id={`${operator.instanceId}-${operatorModule}`} onCheckedChange={(checked) => handleParamChange("conditionals", { ...params.conditionals, moduleDamage: checked })} />
                                                <Label className="ml-2" htmlFor={`${operator.instanceId}-${operatorModule}`}>
                                                    {operatorModule.charAt(0).toUpperCase() + operatorModule.slice(1)}
                                                </Label>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            </CardContent>
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>
        </Card>
    );
}
