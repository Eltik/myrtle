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
import { ChevronDown } from "lucide-react";
import { AnimatePresence, motion } from "framer-motion";

export default function OperatorListItem({ operator, onParamsChange }: OperatorListItemProps) {
    const [params, setParams] = useState<OperatorParams>({});
    const [isOpen, setIsOpen] = useState(false);
    const initializedRef = useRef(false);

    const handleParamChange = (key: keyof OperatorParams, value: OperatorParams[keyof OperatorParams]) => {
        console.log("asdf");
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
                <button className="w-full text-left focus:outline-none" onClick={() => setIsOpen(!isOpen)} aria-expanded={isOpen}>
                    <CardContent className="flex items-center justify-between p-4">
                        <div className="flex items-center">
                            <Image src={`https://raw.githubusercontent.com/ArknightsAssets/ArknightsAssets/cn/assets/torappu/dynamicassets/arts/charavatars/${operator.operatorData.data.id}.png`} alt={operator.operatorData.data.name} width={48} height={48} className="mr-4 rounded-full" />
                            <h3 className="text-lg font-semibold">{operator.operatorData.data.name}</h3>
                        </div>
                        <motion.div animate={{ rotate: isOpen ? 180 : 0 }} transition={{ duration: 0.3 }}>
                            <ChevronDown className="h-4 w-4" />
                        </motion.div>
                    </CardContent>
                </button>
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
                                        {["traitDamage", "talentDamage", "talent2Damage", "skillDamage", "moduleDamage"].map((cond) => (
                                            <div key={cond} className="flex items-center">
                                                <Switch id={`${operator.operatorData.data.id}-${cond}`} onCheckedChange={(checked) => handleParamChange("conditionals", { ...params.conditionals, [cond]: checked })} />
                                                <Label htmlFor={`${operator.operatorData.data.id}-${cond}`} className="ml-2">
                                                    {cond.charAt(0).toUpperCase() + cond.slice(1)}
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
