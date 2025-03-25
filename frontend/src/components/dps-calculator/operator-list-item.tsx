import { useEffect, useRef, useState } from "react";
import type { OperatorParams } from "~/types/impl/api/impl/dps-calculator";
import type { OperatorListItemProps } from "~/types/impl/frontend/impl/dps-calculator";
import { Card, CardContent } from "../ui/card";
import Image from "next/image";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";
import { OperatorRarity } from "~/types/impl/api/static/operator";
import { Input } from "../ui/input";
import { Switch } from "../ui/switch";
import { Label } from "../ui/label";
import { ChevronDown, X } from "lucide-react";
import { AnimatePresence, motion } from "framer-motion";
import { Button } from "../ui/button";

export default function OperatorListItem({ operator, onParamsChange, onRemove }: OperatorListItemProps) {
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
                masteryLevel: operator.operatorData.data.rarity === OperatorRarity.threeStar ? 7 : 9,
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
                    <button className="flex flex-grow items-center justify-between text-left focus:outline-none" onClick={() => setIsOpen(!isOpen)} aria-expanded={isOpen}>
                        <div className="flex items-center">
                            <Image src={`https://raw.githubusercontent.com/ArknightsAssets/ArknightsAssets/cn/assets/torappu/dynamicassets/arts/charavatars/${operator.operatorData.data.id}.png`} alt={operator.operatorData.data.name} width={48} height={48} className="mr-4 rounded-full" />
                            <h3 className="text-lg font-semibold">{operator.operatorData.data.name}</h3>
                        </div>
                        <motion.div animate={{ rotate: isOpen ? 180 : 0 }} transition={{ duration: 0.3 }}>
                            <ChevronDown className="h-4 w-4" />
                        </motion.div>
                    </button>
                    <Button
                        variant="ghost"
                        size="icon"
                        className="ml-2 h-8 w-8 rounded-full"
                        onClick={(e) => {
                            e.stopPropagation();
                            onRemove?.(operator.operatorData.data.id ?? "");
                        }}
                        aria-label={`Remove ${operator.operatorData.data.name}`}
                    >
                        <X className="h-4 w-4" />
                    </Button>
                </div>
                <AnimatePresence initial={false}>
                    {isOpen && (
                        <motion.div
                            initial="collapsed"
                            animate="open"
                            exit="collapsed"
                            variants={{
                                open: { opacity: 1, height: "auto" },
                                collapsed: { opacity: 0, height: 0 },
                            }}
                            transition={{ duration: 0.3, ease: "easeInOut" }}
                        >
                            <CardContent className="p-4 pt-0">
                                <div className="grid grid-cols-2 gap-4">
                                    {operator.operatorData.data.rarity !== OperatorRarity.twoStar && operator.operatorData.data.rarity !== OperatorRarity.oneStar ? (
                                        <>
                                            <Select onValueChange={(value) => handleParamChange("skillIndex", parseInt(value))} defaultValue={String(operator.defaultSkillIndex)}>
                                                <SelectTrigger>
                                                    <SelectValue placeholder="Select skill" />
                                                </SelectTrigger>
                                                <SelectContent>
                                                    {operator.operatorData.data.skills.map((skill, index) => (
                                                        <SelectItem key={index} value={index.toString()} className="truncate">
                                                            S{index + 1} - {skill.static?.levels[0]?.name}
                                                        </SelectItem>
                                                    ))}
                                                </SelectContent>
                                            </Select>
                                            <Select onValueChange={(value) => handleParamChange("masteryLevel", parseInt(value))} defaultValue={operator.operatorData.data.rarity === OperatorRarity.threeStar ? "7" : "9"}>
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
                                                    <Select onValueChange={(value) => handleParamChange("moduleIndex", parseInt(value))} defaultValue={String(operator.defaultModuleIndex)}>
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
                                                    <Select onValueChange={(value) => handleParamChange("moduleLevel", parseInt(value))} defaultValue={"3"}>
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
                                    <h4 className="mb-2 text-sm font-semibold">Buffs</h4>
                                    <div className="grid grid-cols-2 gap-4">
                                        <Input type="number" placeholder="ATK % buff" className="input input-bordered w-full" onChange={(e) => handleParamChange("buffs", { ...params.buffs, atk: parseFloat(e.target.value) })} />
                                        <Input type="number" placeholder="Flat ATK buff" className="input input-bordered w-full" onChange={(e) => handleParamChange("buffs", { ...params.buffs, atkFlat: parseInt(e.target.value) })} />
                                        <Input type="number" placeholder="ASPD buff" className="input input-bordered w-full" onChange={(e) => handleParamChange("buffs", { ...params.buffs, aspd: parseInt(e.target.value) })} />
                                        <Input type="number" placeholder="Fragile debuff" className="input input-bordered w-full" onChange={(e) => handleParamChange("buffs", { ...params.buffs, fragile: parseFloat(e.target.value) })} />
                                    </div>
                                </div>
                                <div className="mt-4">
                                    <h4 className="mb-2 text-sm font-semibold">Conditionals</h4>
                                    <div className="space-y-2">
                                        {operator.talentDamageNames.map((talent) => (
                                            <div key={talent} className="flex items-center">
                                                <Switch id={`${operator.operatorData.data.id}-${talent}`} onCheckedChange={(checked) => handleParamChange("conditionals", { ...params.conditionals, talentDamage: checked })} defaultChecked={operator.talentDamage} />
                                                <Label htmlFor={`${operator.operatorData.data.id}-${talent}`} className="ml-2">
                                                    {talent.charAt(0).toUpperCase() + talent.slice(1)}
                                                </Label>
                                            </div>
                                        ))}
                                        {operator.talent2DamageNames.map((talent) => (
                                            <div key={talent} className="flex items-center">
                                                <Switch id={`${operator.operatorData.data.id}-${talent}`} onCheckedChange={(checked) => handleParamChange("conditionals", { ...params.conditionals, talent2Damage: checked })} defaultChecked={operator.talent2Damage} />
                                                <Label htmlFor={`${operator.operatorData.data.id}-${talent}`} className="ml-2">
                                                    {talent.charAt(0).toUpperCase() + talent.slice(1)}
                                                </Label>
                                            </div>
                                        ))}
                                        {operator.traitDamageNames.map((trait) => (
                                            <div key={trait} className="flex items-center">
                                                <Switch id={`${operator.operatorData.data.id}-${trait}`} onCheckedChange={(checked) => handleParamChange("conditionals", { ...params.conditionals, traitDamage: checked })} defaultChecked={operator.traitDamage} />
                                                <Label htmlFor={`${operator.operatorData.data.id}-${trait}`} className="ml-2">
                                                    {trait.charAt(0).toUpperCase() + trait.slice(1)}
                                                </Label>
                                            </div>
                                        ))}
                                        {operator.skillDamageNames.map((skill) => (
                                            <div key={skill} className="flex items-center">
                                                <Switch id={`${operator.operatorData.data.id}-${skill}`} onCheckedChange={(checked) => handleParamChange("conditionals", { ...params.conditionals, skillDamage: checked })} defaultChecked={operator.skillDamage} />
                                                <Label htmlFor={`${operator.operatorData.data.id}-${skill}`} className="ml-2">
                                                    {skill.charAt(0).toUpperCase() + skill.slice(1)}
                                                </Label>
                                            </div>
                                        ))}
                                        {operator.moduleDamageNames.map((operatorModule) => (
                                            <div key={operatorModule} className="flex items-center">
                                                <Switch id={`${operator.operatorData.data.id}-${operatorModule}`} onCheckedChange={(checked) => handleParamChange("conditionals", { ...params.conditionals, moduleDamage: checked })} defaultChecked={operator.moduleDamage} />
                                                <Label htmlFor={`${operator.operatorData.data.id}-${operatorModule}`} className="ml-2">
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
