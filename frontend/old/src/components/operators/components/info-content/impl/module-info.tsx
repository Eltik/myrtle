import { Info } from "lucide-react";
import { Input } from "~/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";
import { Slider } from "~/components/ui/slider";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/tooltip";
import type { Module, ModuleData } from "~/types/impl/api/static/modules";
import type { Operator } from "~/types/impl/api/static/operator";

export const ModuleInfo = ({
    phaseIndex,
    moduleData,
    modules,
    currentModule,
    setCurrentModule,
    currentModuleLevel,
    setCurrentModuleLevel,
    favorPoint,
    setFavorPoint,
    operator,
    setPotentialRank,
    handleFavorPointChange,
}: {
    phaseIndex: number;
    moduleData:
        | (ModuleData & {
              id: string;
          })[]
        | null;
    modules: Module[];
    currentModule: string;
    setCurrentModule: (module: string) => void;
    currentModuleLevel: number;
    setCurrentModuleLevel: (level: number) => void;
    favorPoint: number;
    setFavorPoint: (point: number) => void;
    operator: Operator;
    setPotentialRank: (rank: number) => void;
    handleFavorPointChange: (point: number) => void;
}) => {
    return (
        <div className="mt-3 flex w-full flex-col gap-4">
            {/* Module Selection Section */}
            {phaseIndex === 2 && moduleData && moduleData.length > 0 && (
                <div className="rounded-md border p-3">
                    <h4 className="mb-2 font-medium text-sm">Module Selection</h4>
                    <div className="flex flex-col gap-3 md:flex-row">
                        <div className="flex flex-col gap-1">
                            <div className="flex items-center gap-1">
                                <label className="text-muted-foreground text-xs">Module Type</label>
                                <TooltipProvider>
                                    <Tooltip>
                                        <TooltipTrigger asChild>
                                            <Info className="h-3 w-3 cursor-help text-muted-foreground" />
                                        </TooltipTrigger>
                                        <TooltipContent>
                                            <p>Select a module to see how it affects the operator&apos;s stats</p>
                                        </TooltipContent>
                                    </Tooltip>
                                </TooltipProvider>
                            </div>
                            <Select
                                defaultValue={currentModule !== "" ? currentModule : (modules[modules.length - 1]?.id ?? "")}
                                onValueChange={(value) => {
                                    setCurrentModule(value);
                                    setCurrentModuleLevel(moduleData?.find((module) => module.id === value)?.phases?.[(moduleData?.find((module) => module.id === value)?.phases?.length ?? 0) - 1]?.equipLevel ?? 0);
                                }}
                            >
                                <SelectTrigger className="md:w-[180px]">
                                    <SelectValue placeholder="Select a Module" />
                                </SelectTrigger>
                                <SelectContent>
                                    {modules.map((module, index) => (
                                        <SelectItem key={index} value={module.id ?? ""}>
                                            {module.typeName1 && module.typeName2 ? `${module.typeName1}-${module.typeName2}` : module.uniEquipName}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>

                        {phaseIndex === 2 && currentModule.length && moduleData?.find((module) => module.id === currentModule)?.phases !== undefined && (
                            <div className="flex flex-col gap-1">
                                <div className="flex items-center gap-1">
                                    <label className="text-muted-foreground text-xs">Module Level</label>
                                    <TooltipProvider>
                                        <Tooltip>
                                            <TooltipTrigger asChild>
                                                <Info className="h-3 w-3 cursor-help text-muted-foreground" />
                                            </TooltipTrigger>
                                            <TooltipContent>
                                                <p>Select the level of the module to see its effects</p>
                                            </TooltipContent>
                                        </Tooltip>
                                    </TooltipProvider>
                                </div>
                                <Select
                                    defaultValue={currentModuleLevel !== 0 ? `${module.id}_${currentModuleLevel}` : `${module.id}_${moduleData?.find((module) => module.id === currentModule)?.phases?.[(moduleData?.find((module) => module.id === currentModule)?.phases?.length ?? 0) - 1]?.equipLevel ?? 0}`}
                                    onValueChange={(value) => {
                                        if (!Number.isNaN(parseInt(value.split("_")[1] ?? "0", 10))) {
                                            setCurrentModuleLevel(parseInt(value.split("_")[1] ?? "0", 10));
                                        }
                                    }}
                                >
                                    <SelectTrigger className="md:w-[180px]">
                                        <SelectValue placeholder="Select a Module Level" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {moduleData
                                            ?.find((module) => module.id === currentModule)
                                            ?.phases.map((phase, index) => (
                                                <SelectItem key={index} value={`${module.id}_${phase.equipLevel}`}>
                                                    Level {phase.equipLevel}
                                                </SelectItem>
                                            ))}
                                    </SelectContent>
                                </Select>
                            </div>
                        )}
                    </div>
                </div>
            )}

            {/* Potential and Trust Section */}
            <div className="rounded-md border p-3">
                <h4 className="mb-2 font-medium text-sm">Operator Attributes</h4>
                <div className="flex flex-col gap-3 md:flex-row md:items-end">
                    <div className="flex flex-col gap-1">
                        <div className="flex items-center gap-1">
                            <label className="text-muted-foreground text-xs">Potential Rank</label>
                            <TooltipProvider>
                                <Tooltip>
                                    <TooltipTrigger asChild>
                                        <Info className="h-3 w-3 cursor-help text-muted-foreground" />
                                    </TooltipTrigger>
                                    <TooltipContent>
                                        <p>Select the potential rank to see how it affects the operator&apos;s stats</p>
                                    </TooltipContent>
                                </Tooltip>
                            </TooltipProvider>
                        </div>
                        <Select
                            defaultValue={"potential_0"}
                            onValueChange={(value) => {
                                setPotentialRank(parseInt(value.split("_")[1] ?? "0", 10));
                            }}
                        >
                            <SelectTrigger className="md:w-[180px]">
                                <SelectValue placeholder="Potential Rank" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value={"potential_0"}>No Potential</SelectItem>
                                {operator.potentialRanks.map((rank, index) => (
                                    <SelectItem key={index} value={`potential_${index + 1}`}>
                                        Potential {index + 1} - {rank.description}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>

                    <div className="flex flex-col gap-1">
                        <div className="flex items-center gap-1">
                            <label className="text-muted-foreground text-xs">Trust Level</label>
                            <TooltipProvider>
                                <Tooltip>
                                    <TooltipTrigger asChild>
                                        <Info className="h-3 w-3 cursor-help text-muted-foreground" />
                                    </TooltipTrigger>
                                    <TooltipContent>
                                        <p>Adjust the trust level to see how it affects the operator&apos;s stats</p>
                                    </TooltipContent>
                                </Tooltip>
                            </TooltipProvider>
                        </div>
                        <div className="max-w-md">
                            <Slider max={200} min={0} onValueChange={(value) => setFavorPoint(value[0] ?? 0)} step={1} value={[favorPoint]} />
                            <div className="mt-2 flex flex-row items-center gap-2 text-sm">
                                <Input className="w-20" max={200} min={0} onChange={(e) => handleFavorPointChange(Number(e.target.value))} type="number" value={favorPoint} />
                                <span className="text-muted-foreground">Trust</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};
