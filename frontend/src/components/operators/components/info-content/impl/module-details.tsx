import { useState } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";
import { Button } from "~/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/collapsible";
import { Badge } from "~/components/ui/badge";
import { Separator } from "~/components/ui/separator";
import type { Module, ModuleData } from "~/types/impl/api/static/modules";
import { ModuleTarget } from "~/types/impl/api/static/modules";
import { AnimatePresence, motion } from "framer-motion";
import Image from "next/image";
import { descriptionToHtml } from "~/helper/descriptionParser";
import { parsePhase } from "~/helper";
import { OperatorPhase } from "~/types/impl/api/static/operator";

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
            <div className="space-y-4">
                {/* Module Header */}
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <Image src={currentModuleInfo.image ?? ""} alt={currentModuleInfo.uniEquipName} className="h-24 w-24 rounded-md" width={96} height={96} />
                        <div>
                            <h3 className="text-lg font-semibold">{currentModuleInfo.uniEquipName}</h3>
                            <div className="flex gap-1">
                                <Badge variant="outline">{currentModuleInfo.typeName1}</Badge>
                                {currentModuleInfo.typeName2 && <Badge variant="outline">{currentModuleInfo.typeName2}</Badge>}
                            </div>
                        </div>
                    </div>
                    <Badge variant="secondary" className="capitalize" style={{ backgroundColor: currentModuleInfo.equipShiningColor }}>
                        {currentModuleInfo.type.toLowerCase()}
                    </Badge>
                </div>

                {/* Module Description */}
                <Collapsible open={isDescriptionExpanded} onOpenChange={setIsDescriptionExpanded}>
                    <div className="flex items-center gap-2">
                        <CollapsibleTrigger asChild>
                            <Button variant="ghost" size="sm" className="flex items-center gap-2 p-2">
                                {isDescriptionExpanded ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
                                <h4 className="text-sm font-medium">Description</h4>
                            </Button>
                        </CollapsibleTrigger>
                    </div>
                    <CollapsibleContent>
                        <p className="mt-2 text-sm text-muted-foreground">{currentModuleInfo.uniEquipDesc}</p>
                    </CollapsibleContent>
                </Collapsible>

                <Separator />

                {/* Module Requirements */}
                <div className="space-y-2">
                    <h4 className="font-medium">Requirements</h4>
                    <div className="grid grid-cols-2 gap-4 text-sm">
                        <div>
                            <span className="text-muted-foreground">Show Level:</span> <span>{currentModuleInfo.showLevel}</span>
                        </div>
                        <div>
                            <span className="text-muted-foreground">Unlock Level:</span> <span>{currentModuleInfo.unlockLevel}</span>
                        </div>
                        <div className="flex flex-row items-center gap-1">
                            <span className="text-muted-foreground">Show Phase:</span>
                            <Image
                                src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${parsePhase(currentModuleInfo.showEvolvePhase) === OperatorPhase.ELITE_0 ? 0 : parsePhase(currentModuleInfo.showEvolvePhase) === OperatorPhase.ELITE_1 ? 1 : 2}.png`}
                                width={25}
                                height={25}
                                alt="Promotion"
                                style={{
                                    maxWidth: "100%",
                                    height: "auto",
                                    objectFit: "contain",
                                }}
                            />
                        </div>
                        <div className="flex flex-row items-center gap-1">
                            <span className="text-muted-foreground">Unlock Phase:</span>
                            <Image
                                src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${parsePhase(currentModuleInfo.unlockEvolvePhase) === OperatorPhase.ELITE_0 ? 0 : parsePhase(currentModuleInfo.unlockEvolvePhase) === OperatorPhase.ELITE_1 ? 1 : 2}.png`}
                                width={25}
                                height={25}
                                alt="Promotion"
                                style={{
                                    maxWidth: "100%",
                                    height: "auto",
                                    objectFit: "contain",
                                }}
                            />
                        </div>
                        <div>
                            <span className="text-muted-foreground">Trust Required:</span> <span>{currentModuleInfo.unlockFavorPoint ?? "N/A"}</span>
                        </div>
                    </div>
                </div>

                <Separator />

                {/* Module Phases */}
                <div className="space-y-4">
                    <h4 className="font-medium">Module Phases</h4>
                    {currentModuleData.phases.map((phase, phaseIndex) => (
                        <Collapsible key={phaseIndex} open={openSections[`phase-${phaseIndex}`]} onOpenChange={() => toggleSection(`phase-${phaseIndex}`)}>
                            <CollapsibleTrigger asChild>
                                <Button variant="outline" className="w-full justify-between">
                                    <span>
                                        Phase {phaseIndex + 1} (Level {phase.equipLevel})
                                    </span>
                                    {openSections[`phase-${phaseIndex}`] ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
                                </Button>
                            </CollapsibleTrigger>
                            <CollapsibleContent>
                                <AnimatePresence>
                                    <motion.div initial={{ height: 0, opacity: 0 }} animate={{ height: "auto", opacity: 1 }} exit={{ height: 0, opacity: 0 }} transition={{ duration: 0.2 }} className="space-y-4 p-4">
                                        {/* Attributes */}
                                        {phase.attributeBlackboard.length > 0 && (
                                            <div>
                                                <h5 className="mb-2 font-medium">Attributes</h5>
                                                <div className="grid grid-cols-2 gap-2">
                                                    {phase.attributeBlackboard.map((attr, i) => (
                                                        <div key={i} className="text-sm">
                                                            <span className="text-muted-foreground">{attr.key}:</span> <span>+{attr.value}</span>
                                                        </div>
                                                    ))}
                                                </div>
                                            </div>
                                        )}

                                        {/* Parts */}
                                        {phase.parts.map((part, partIndex) => {
                                            // Check if we have any valid candidates to display
                                            const hasTalentCandidates = part.addOrOverrideTalentDataBundle.candidates?.some((candidate) => candidate.upgradeDescription || candidate.description);

                                            const hasTraitCandidates = part.overrideTraitDataBundle.candidates?.some((candidate) => candidate.additionalDescription);

                                            // Only render if we have content to show
                                            if (!hasTalentCandidates && !hasTraitCandidates) {
                                                return null;
                                            }

                                            return (
                                                <div key={partIndex} className="space-y-2">
                                                    <h5 className="font-medium">{part.target === ModuleTarget.TRAIT ? "Trait Changes" : "Talent Changes"}</h5>

                                                    {hasTalentCandidates &&
                                                        part.addOrOverrideTalentDataBundle.candidates?.map((candidate, i) => {
                                                            // Skip candidates without description
                                                            if (!candidate.upgradeDescription && !candidate.description) {
                                                                return null;
                                                            }

                                                            return (
                                                                <div key={i} className="rounded-md bg-muted p-2">
                                                                    {candidate.name && <div className="font-medium">{candidate.name}</div>}
                                                                    <p
                                                                        className="text-sm text-muted-foreground"
                                                                        dangerouslySetInnerHTML={{
                                                                            __html: descriptionToHtml(candidate.upgradeDescription ?? candidate.description ?? "", candidate.blackboard ?? []),
                                                                        }}
                                                                    ></p>
                                                                </div>
                                                            );
                                                        })}

                                                    {hasTraitCandidates &&
                                                        part.overrideTraitDataBundle.candidates?.map((candidate, i) => {
                                                            // Skip candidates without additionalDescription
                                                            if (!candidate.additionalDescription) {
                                                                return null;
                                                            }

                                                            return (
                                                                <div key={i} className="rounded-md bg-muted p-2">
                                                                    <p
                                                                        className="text-sm text-muted-foreground"
                                                                        dangerouslySetInnerHTML={{
                                                                            __html: descriptionToHtml(candidate.additionalDescription ?? "", candidate.blackboard ?? []),
                                                                        }}
                                                                    ></p>
                                                                </div>
                                                            );
                                                        })}
                                                </div>
                                            );
                                        })}
                                    </motion.div>
                                </AnimatePresence>
                            </CollapsibleContent>
                        </Collapsible>
                    ))}
                </div>
            </div>
        </div>
    );
}
