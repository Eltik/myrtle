import { useState } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";
import { Button } from "~/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/collapsible";
import { Badge } from "~/components/ui/badge";
import { Separator } from "~/components/ui/separator";
import type { Module, ModuleData } from "~/types/impl/api/static/modules";
import { AnimatePresence, motion } from "framer-motion";

interface ModuleDetailsProps {
    currentModule: string;
    modules: Module[];
    moduleData: (ModuleData & { id: string })[] | null;
}

export function ModuleDetails({ currentModule, modules, moduleData }: ModuleDetailsProps) {
    const [openSections, setOpenSections] = useState<Record<string, boolean>>({});
    const [isDescriptionExpanded, setIsDescriptionExpanded] = useState(false);

    const toggleSection = (section: string) => {
        setOpenSections((prev) => ({ ...prev, [section]: !prev[section] }));
    };

    const currentModuleData = moduleData?.find((m) => m.id === currentModule);
    const currentModuleInfo = modules.find((m) => m.id === currentModule);

    if (!currentModuleData || !currentModuleInfo) {
        return null;
    }

    return (
        <div className="mt-2 rounded-md px-4 py-2">
            <div className="flex flex-col gap-3">
                <h2 className="text-lg font-bold">{currentModuleInfo.uniEquipName}</h2>
                <Separator />
            </div>
            <div className="mt-4 space-y-4">
                <div className="flex flex-col gap-2">
                    <h3 className="text-lg font-semibold">Description</h3>
                    <div className="relative rounded-md border p-2">
                        <AnimatePresence initial={false}>
                            <motion.div initial={{ height: "80px" }} animate={{ height: isDescriptionExpanded ? "auto" : "80px" }} exit={{ height: "80px" }} transition={{ duration: 0.3 }} className="overflow-hidden">
                                <p className="text-sm">{currentModuleInfo.uniEquipDesc}</p>
                            </motion.div>
                        </AnimatePresence>
                        {!isDescriptionExpanded && <div className="absolute bottom-0 left-0 right-0 h-16 bg-gradient-to-t from-background to-transparent" />}
                    </div>
                    <span onClick={() => setIsDescriptionExpanded(!isDescriptionExpanded)} className="flex cursor-pointer flex-row items-center self-start text-sm transition-all duration-150 hover:text-muted-foreground">
                        {isDescriptionExpanded ? "Show Less" : "Show More"}
                        <ChevronDown className={`ml-auto h-5 w-5 transition-transform ${isDescriptionExpanded ? "rotate-180" : ""}`} />
                    </span>
                </div>
                <Separator />
                <div>
                    <h3 className="mb-2 text-lg font-semibold">Details</h3>
                    <div className="grid grid-cols-2 gap-2">
                        <Badge variant="outline">Type: {currentModuleInfo.typeName1}</Badge>
                        {currentModuleInfo.typeName2 && <Badge variant="outline">Subtype: {currentModuleInfo.typeName2}</Badge>}
                        <Badge variant="outline">Unlock Level: {currentModuleInfo.unlockLevel}</Badge>
                        <Badge variant="outline">Trust Requirement: {currentModuleInfo.unlockFavorPoint}</Badge>
                    </div>
                </div>
                <Separator />
                <div>
                    <h3 className="mb-2 text-lg font-semibold">Module Levels</h3>
                    {currentModuleData.phases.map((phase, index) => (
                        <Collapsible key={index} open={openSections[`phase_${index}`]} onOpenChange={() => toggleSection(`phase_${index}`)}>
                            <CollapsibleTrigger asChild>
                                <Button variant="ghost" className={`w-full justify-between ${openSections[`phase_${index}`] ? "border bg-primary-foreground" : ""}`}>
                                    <span>Level {phase.equipLevel}</span>
                                    {openSections[`phase_${index}`] ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
                                </Button>
                            </CollapsibleTrigger>
                            <CollapsibleContent className="space-y-2">
                                {phase.parts.map((part, partIndex) => (
                                    <div key={partIndex} className="ml-8">
                                        <h4 className="text-base font-semibold">Part {partIndex + 1}</h4>
                                        <p className="text-sm">Target: {part.target}</p>
                                        {part.addOrOverrideTalentDataBundle?.candidates?.map((candidate, candIndex) => (
                                            <div key={candIndex} className="ml-4 mt-2">
                                                <p className="text-sm font-medium">{candidate.name}</p>
                                                <p className="text-sm text-muted-foreground">{candidate.description}</p>
                                            </div>
                                        ))}
                                    </div>
                                ))}
                                {phase.attributeBlackboard.length > 0 && (
                                    <div className="ml-8">
                                        <h4 className="font-semibold">Attribute Changes</h4>
                                        <div className="mt-2 grid grid-cols-2 gap-2">
                                            {phase.attributeBlackboard.map((attr, attrIndex) => (
                                                <Badge key={attrIndex} variant="secondary">
                                                    {attr.key}: {attr.value}
                                                </Badge>
                                            ))}
                                        </div>
                                    </div>
                                )}
                            </CollapsibleContent>
                        </Collapsible>
                    ))}
                </div>
            </div>
        </div>
    );
}
