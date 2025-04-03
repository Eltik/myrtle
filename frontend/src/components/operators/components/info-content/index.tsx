/* eslint-disable react-hooks/exhaustive-deps */
import { ChevronDown } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { LevelSlider } from "~/components/operators/components/info-content/impl/level-slider";
import { Separator } from "~/components/ui/separator";
import { getOperatorAttributeStats } from "~/helper/getAttributeStats";
import type { Module, ModuleData } from "~/types/impl/api/static/modules";
import { type Operator } from "~/types/impl/api/static/operator";
import type { Range } from "~/types/impl/api/static/ranges";
import OperatorRange from "../operator-range";
import { Button } from "~/components/ui/button";
import { ModuleDetails } from "./impl/module-details";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/collapsible";
import { TopInfoContent } from "./impl/top-info";
import { TopDescription } from "./impl/top-description";
import { ModuleInfo } from "./impl/module-info";
import { Stats } from "./impl/stats";
import { Handbook } from "./impl/handbook";

// Credit to:
// https://aceship.github.io/AN-EN-Tags/akhrchars.html?opname=Projekt_Red
// https://sanitygone.help/operators/gavial-the-invincible#page-content

function InfoContent({ operator }: { operator: Operator }) {
    const [showControls, setShowControls] = useState<boolean>(true); // Show controls for the operator by default
    const [isModuleDetailsExpanded, setIsModuleDetailsExpanded] = useState(true);
    const [isHandbookExpanded, setIsHandbookExpanded] = useState(true);

    const [attributeStats, setAttributeStats] = useState<Operator["phases"][number]["attributesKeyFrames"][number]["data"] | null>(null); // Attribute stats to display

    const [moduleData, setModuleData] = useState<(ModuleData & { id: string })[] | null>(null); // All modules of the operator
    const [modules, setModules] = useState<Module[]>([]); // Modules of the operator
    const [currentModule, setCurrentModule] = useState<string>(""); // Current module selected to calculate the attribute stats for
    const [currentModuleLevel, setCurrentModuleLevel] = useState<number>(0); // Current module level selected to calculate the attribute stats for

    const [phaseIndex, setPhaseIndex] = useState<number>(operator.phases.length - 1); // Phase index of the operator
    const [level, setLevel] = useState<number>(1); // Level of the operator
    const [favorPoint, setFavorPoint] = useState<number>(100); // Favor point of the operator
    const [potentialRank, setPotentialRank] = useState<number>(0); // Potential rank of the operator

    const [ranges, setRanges] = useState<Range[] | null>(null); // Range of the operator
    const [currentRangeId, setCurrentRangeId] = useState<string>(""); // Current range selected to display

    // Memoize the current range
    const currentRange = useMemo(() => {
        return ranges?.find((range) => range.id === currentRangeId) ?? null;
    }, [ranges, currentRangeId]);

    /**
     * @description On first load, fetch modules & ranges.
     */
    useEffect(() => {
        void fetchModules();

        setLevel(operator.phases[operator.phases.length - 1]?.maxLevel ?? 1);
    }, [operator]);

    /**
     * @description On changing the level, phase, module, trust, etc.
     */
    useEffect(() => {
        if (phaseIndex !== 2) {
            setCurrentModule("");
            setCurrentModuleLevel(0);
        } else if (currentModule === "") {
            setCurrentModule(modules[modules.length - 1]?.id ?? "");
            setCurrentModuleLevel(moduleData?.find((module) => module.id === modules[modules.length - 1]?.id)?.phases[(moduleData?.find((module) => module.id === modules[modules.length - 1]?.id)?.phases ?? []).length - 1]?.equipLevel ?? 0);
        }

        const operatorPhase = operator.phases[phaseIndex];
        if (operatorPhase) {
            if ((ranges?.find((range) => range.id === operatorPhase.rangeId) ?? null) !== null) {
                setCurrentRangeId(operatorPhase.rangeId);
            } else {
                console.log("Operator phase range not found:", operatorPhase.rangeId);
            }
        }

        const operatorModule = moduleData?.find((module) => module.id === currentModule);
        for (const phase of operatorModule?.phases ?? []) {
            if (phase.equipLevel === currentModuleLevel) {
                for (const part of phase.parts) {
                    for (const candidate of part.addOrOverrideTalentDataBundle?.candidates ?? []) {
                        if (candidate.rangeId && candidate.rangeId.length > 0) {
                            if ((ranges?.find((range) => range.id === candidate.rangeId) ?? null) !== null) {
                                setCurrentRangeId(candidate.rangeId);
                            } else {
                                console.log("Module range not found:", candidate.rangeId);
                            }
                        }
                    }
                }
            }
        }

        fetchAttributeStats(
            {
                phaseIndex: phaseIndex,
                favorPoint: favorPoint,
                potentialRank: potentialRank,
                moduleId: currentModule,
                moduleLevel: currentModuleLevel,
            },
            level,
            currentModule,
        );
    }, [ranges, level, phaseIndex, favorPoint, potentialRank, currentModuleLevel, currentModule]);

    /**
     * @description Fetches all info on modules, module data, and ranges.
     */
    async function fetchModules() {
        const data = {
            modules: operator.modules,
        };

        setModules(data.modules);

        const moduleData: (ModuleData & { id: string })[] = [];
        for (const operatorModule of data.modules) {
            const details = fetchModuleData(operatorModule.uniEquipId ?? "");
            if (!details) continue;

            moduleData.push({ ...details, id: operatorModule.uniEquipId ?? "" });
        }

        setModuleData(moduleData);

        if (data.modules[data.modules.length - 1]?.id !== undefined) {
            setCurrentModule(data.modules[data.modules.length - 1]!.id!);
            setCurrentModuleLevel(moduleData?.find((module) => module.id === data.modules[data.modules.length - 1]?.id)?.phases[(moduleData?.find((module) => module.id === data.modules[data.modules.length - 1]?.id)?.phases ?? []).length - 1]?.equipLevel ?? 0);
        }

        for (const operatorModule of moduleData) {
            if (operatorModule.phases !== undefined) {
                for (const phase of operatorModule.phases) {
                    for (const part of phase.parts) {
                        for (const candidate of part.addOrOverrideTalentDataBundle?.candidates ?? []) {
                            if (candidate.rangeId) {
                                const range = await fetchRange(candidate.rangeId);
                                if (!range) continue;

                                setRanges([...(ranges ?? []), range]);

                                // Since modules modify ranges, set the current range
                                // to whatever the last module's range is
                                if (ranges?.find((range) => range.id === candidate.rangeId) !== undefined) {
                                    setCurrentRangeId(range.id);
                                }
                            }
                        }
                    }
                }
            }
        }

        await fetchRanges();
    }

    async function fetchRanges() {
        const promises = [];
        const allRanges: (Range & { id: string })[] = ranges ? [...ranges] : [];

        for (const phase of operator.phases) {
            const exists = allRanges.find((range) => range.id === phase.rangeId);
            if (exists || phase.rangeId.length === 0) {
                continue;
            }

            const promise = new Promise<void>((resolve) => {
                fetchRange(phase.rangeId)
                    .then((data) => {
                        allRanges.push({ ...data, id: phase.rangeId });
                        resolve();
                    })
                    .catch((err) => {
                        console.error("Error fetching module data:", err);
                        resolve();
                    });
            });
            promises.push(promise);
        }

        await Promise.all(promises);

        setRanges(allRanges);
    }

    async function fetchRange(rangeId: string) {
        const data = (await (
            await fetch("/api/static", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    type: "ranges",
                    id: rangeId,
                }),
            })
        ).json()) as { data: Range };
        return data.data;
    }

    const fetchAttributeStats = (
        metadata: {
            phaseIndex: number;
            favorPoint: number;
            potentialRank: number;
            moduleId: string;
            moduleLevel: number;
        },
        level: number,
        moduleId: string,
    ) => {
        const battleEquip = {
            [moduleId]: moduleData?.find((module) => module.id === moduleId) ?? null,
        };

        if (battleEquip[moduleId] !== null) {
            const stats = getOperatorAttributeStats(operator, metadata, level, battleEquip as Record<string, ModuleData>);
            setAttributeStats(stats);
        } else {
            const stats = getOperatorAttributeStats(operator, metadata, level);
            setAttributeStats(stats);
        }
    };

    const fetchModuleData = (moduleId: string): ModuleData | undefined => {
        const moduleData = operator.modules.find((module) => module.uniEquipId === moduleId)?.data;
        return moduleData;
    };

    const handleLevelChange = (newLevel: number) => {
        setLevel(newLevel);
    };

    const handleFavorPointChange = (value: number) => {
        if (value >= 0 && value <= 200) {
            setFavorPoint(value);
        }
    };

    return (
        <div className="w-full overflow-x-hidden">
            <div className="p-2 px-4 backdrop-blur-2xl">
                <span className="text-xl font-bold md:text-3xl">Operator Info</span>
            </div>
            <Separator />
            <div className="mx-auto max-w-4xl px-3 py-4 md:p-4">
                <TopInfoContent operator={operator} />
                <div className="mt-3 block w-full">
                    {/**
                     * @description Operator's description
                     */}
                    <div className="flex flex-col gap-2">
                        <TopDescription operator={operator} />
                    </div>

                    {/* Controls Section with improved visibility and organization */}
                    <div className="mt-4 rounded-md border p-4">
                        <div className="mb-2 flex items-center justify-between">
                            <h3 className="text-lg font-semibold">Operator Controls</h3>
                            <Button variant={"outline"} onClick={() => setShowControls(!showControls)} className="flex flex-row items-center" size="sm">
                                {showControls ? "Hide" : "Show"} Controls
                                <ChevronDown className={`ml-auto h-4 w-4 transition-transform ${showControls ? "rotate-180" : ""}`} />
                            </Button>
                        </div>

                        <div className={`overflow-hidden transition-all duration-300 ${showControls ? "max-h-[1000px] opacity-100" : "max-h-0 opacity-0"}`}>
                            <div className="mb-4 rounded-md bg-muted/30 p-3 text-sm">
                                <p>Adjust these controls to see how the operator&apos;s stats change at different levels, with different modules, and at various trust levels.</p>
                            </div>

                            <ModuleInfo phaseIndex={phaseIndex} moduleData={moduleData} modules={modules} currentModule={currentModule} setCurrentModule={setCurrentModule} currentModuleLevel={currentModuleLevel} setCurrentModuleLevel={setCurrentModuleLevel} favorPoint={favorPoint} setFavorPoint={setFavorPoint} operator={operator} setPotentialRank={setPotentialRank} handleFavorPointChange={handleFavorPointChange} />

                            <div className="mt-3 max-w-md">
                                <div className="mb-2 text-sm font-medium">Operator Level</div>
                                <LevelSlider phaseIndex={phaseIndex} maxLevels={operator.phases.map((phase) => phase.maxLevel)} onLevelChange={handleLevelChange} />
                            </div>
                        </div>
                    </div>

                    <Stats operator={operator} attributeStats={attributeStats} setPhaseIndex={setPhaseIndex} setLevel={setLevel} />
                </div>
                <div className="mt-2 w-full">
                    <div className="mt-2">
                        <h2 className="text-md font-bold md:text-lg">Tags</h2>
                        <div className="flex flex-wrap gap-2 text-sm md:text-base">
                            {operator.tagList.map((tag, index) => (
                                <span key={index} className="rounded-md bg-muted p-1 px-2">
                                    {tag}
                                </span>
                            ))}
                        </div>
                    </div>
                    <div className="mt-2">
                        <h2 className="text-md font-bold md:text-lg">Range</h2>
                        {currentRange ? (
                            <OperatorRange range={currentRange} key={currentRangeId} />
                        ) : (
                            <>
                                <div className="flex flex-row items-center gap-2 text-sm md:text-base">
                                    <span className="text-muted-foreground">No range data available.</span>
                                </div>
                            </>
                        )}
                    </div>
                    {currentModule.length > 0 && (
                        <div className="mt-4">
                            <Collapsible defaultOpen={isModuleDetailsExpanded} onOpenChange={() => setIsModuleDetailsExpanded(!isModuleDetailsExpanded)}>
                                <CollapsibleTrigger asChild>
                                    <div className="flex cursor-pointer flex-row items-center rounded-md px-2 py-1 transition-all duration-150 hover:bg-primary-foreground">
                                        <h2 className="text-lg font-bold">Module Details</h2>
                                        <ChevronDown className={`ml-auto h-5 w-5 transition-transform ${isModuleDetailsExpanded ? "rotate-180" : ""}`} />
                                    </div>
                                </CollapsibleTrigger>
                                <CollapsibleContent>
                                    <div className="mt-2 rounded-md border">
                                        <ModuleDetails currentModule={currentModule} modules={modules} moduleData={moduleData} />
                                    </div>
                                </CollapsibleContent>
                            </Collapsible>
                        </div>
                    )}

                    <Handbook operator={operator} isHandbookExpanded={isHandbookExpanded} setIsHandbookExpanded={setIsHandbookExpanded} />
                </div>
            </div>
        </div>
    );
}

export default InfoContent;
