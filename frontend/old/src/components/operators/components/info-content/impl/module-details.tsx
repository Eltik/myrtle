import { AnimatePresence, motion } from "framer-motion";
import { ChevronDown, ChevronRight } from "lucide-react";
import Image from "next/image";
import { useState } from "react";
import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/collapsible";
import { Separator } from "~/components/ui/separator";
import { parsePhase } from "~/helper";
import { descriptionToHtml } from "~/helper/descriptionParser";
import { getCDNURL } from "~/lib/cdn";
import type { Module, ModuleData } from "~/types/impl/api/static/modules";
import { ModuleTarget } from "~/types/impl/api/static/modules";
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
                        <Image alt={currentModuleInfo.uniEquipName} className="h-24 w-24 rounded-md" height={96} src={currentModuleInfo.image ? getCDNURL(currentModuleInfo.image) : getCDNURL(`equip/${currentModuleInfo.uniEquipIcon}.png`)} width={96} />
                        <div>
                            <h3 className="font-semibold text-lg">{currentModuleInfo.uniEquipName}</h3>
                            <div className="flex gap-1">
                                <Badge variant="outline">{currentModuleInfo.typeName1}</Badge>
                                {currentModuleInfo.typeName2 && <Badge variant="outline">{currentModuleInfo.typeName2}</Badge>}
                            </div>
                        </div>
                    </div>
                    <Badge className="capitalize" style={{ backgroundColor: currentModuleInfo.equipShiningColor }} variant="secondary">
                        {currentModuleInfo.type.toLowerCase()}
                    </Badge>
                </div>

                {/* Module Description */}
                <Collapsible onOpenChange={setIsDescriptionExpanded} open={isDescriptionExpanded}>
                    <div className="flex items-center gap-2">
                        <CollapsibleTrigger asChild>
                            <Button className="flex items-center gap-2 p-2" size="sm" variant="ghost">
                                {isDescriptionExpanded ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
                                <h4 className="font-medium text-sm">Description</h4>
                            </Button>
                        </CollapsibleTrigger>
                    </div>
                    <CollapsibleContent>
                        <p className="mt-2 text-muted-foreground text-sm">{currentModuleInfo.uniEquipDesc}</p>
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
                                alt="Promotion"
                                height={25}
                                src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${parsePhase(currentModuleInfo.showEvolvePhase) === OperatorPhase.ELITE_0 ? 0 : parsePhase(currentModuleInfo.showEvolvePhase) === OperatorPhase.ELITE_1 ? 1 : 2}.png`}
                                style={{
                                    maxWidth: "100%",
                                    height: "auto",
                                    objectFit: "contain",
                                }}
                                width={25}
                            />
                        </div>
                        <div className="flex flex-row items-center gap-1">
                            <span className="text-muted-foreground">Unlock Phase:</span>
                            <Image
                                alt="Promotion"
                                height={25}
                                src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${parsePhase(currentModuleInfo.unlockEvolvePhase) === OperatorPhase.ELITE_0 ? 0 : parsePhase(currentModuleInfo.unlockEvolvePhase) === OperatorPhase.ELITE_1 ? 1 : 2}.png`}
                                style={{
                                    maxWidth: "100%",
                                    height: "auto",
                                    objectFit: "contain",
                                }}
                                width={25}
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
                        <Collapsible key={phaseIndex} onOpenChange={() => toggleSection(`phase-${phaseIndex}`)} open={openSections[`phase-${phaseIndex}`]}>
                            <CollapsibleTrigger asChild>
                                <Button className="w-full justify-between" variant="outline">
                                    <span>
                                        Phase {phaseIndex + 1} (Level {phase.equipLevel})
                                    </span>
                                    {openSections[`phase-${phaseIndex}`] ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
                                </Button>
                            </CollapsibleTrigger>
                            <CollapsibleContent>
                                <AnimatePresence>
                                    <motion.div animate={{ height: "auto", opacity: 1 }} className="space-y-4 p-4" exit={{ height: 0, opacity: 0 }} initial={{ height: 0, opacity: 0 }} transition={{ duration: 0.2 }}>
                                        {/* Attributes */}
                                        {phase.attributeBlackboard.length > 0 && (
                                            <div>
                                                <h5 className="mb-2 font-medium">Attributes</h5>
                                                <div className="grid grid-cols-2 gap-2">
                                                    {phase.attributeBlackboard.map((attr, i) => (
                                                        <div className="text-sm" key={i}>
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
                                                <div className="space-y-2" key={partIndex}>
                                                    <h5 className="font-medium">{part.target === ModuleTarget.TRAIT ? "Trait Changes" : "Talent Changes"}</h5>

                                                    {hasTalentCandidates &&
                                                        part.addOrOverrideTalentDataBundle.candidates?.map((candidate, i) => {
                                                            // Skip candidates without description
                                                            if (!candidate.upgradeDescription && !candidate.description) {
                                                                return null;
                                                            }

                                                            return (
                                                                <div className="rounded-md bg-muted p-2" key={i}>
                                                                    {candidate.name && <div className="font-medium">{candidate.name}</div>}
                                                                    <p
                                                                        className="text-muted-foreground text-sm"
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
                                                                <div className="rounded-md bg-muted p-2" key={i}>
                                                                    <p
                                                                        className="text-muted-foreground text-sm"
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
