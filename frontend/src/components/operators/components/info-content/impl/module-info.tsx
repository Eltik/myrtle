import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";
import { Slider } from "~/components/ui/slider";
import { Input } from "~/components/ui/input";
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
        <>
            <div className="mt-3 flex w-full flex-col gap-4">
                <div className={`flex flex-col justify-between gap-2 md:flex-row ${phaseIndex === 2 && moduleData && moduleData.length > 0 ? "" : "md:justify-end"}`}>
                    <div className="flex flex-col gap-1 md:flex-row">
                        {phaseIndex === 2 && moduleData && moduleData.length > 0 ? (
                            <Select
                                onValueChange={(value) => {
                                    setCurrentModule(value);
                                    setCurrentModuleLevel(moduleData?.find((module) => module.id === value)?.phases?.[(moduleData?.find((module) => module.id === value)?.phases?.length ?? 0) - 1]?.equipLevel ?? 0);
                                }}
                                defaultValue={currentModule !== "" ? currentModule : (modules[modules.length - 1]?.id ?? "")}
                            >
                                <SelectTrigger className="md:w-[180px]">
                                    <SelectValue placeholder="Select a Module" />
                                </SelectTrigger>
                                <SelectContent>
                                    {modules.map((module, index) => (
                                        <SelectItem value={module.id ?? ""} key={index}>
                                            {module.typeName1 && module.typeName2 ? `${module.typeName1}-${module.typeName2}` : module.uniEquipName}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        ) : (
                            <></>
                        )}
                        {phaseIndex === 2 && currentModule.length && moduleData?.find((module) => module.id === currentModule)?.phases !== undefined ? (
                            <Select
                                onValueChange={(value) => {
                                    if (!isNaN(parseInt(value.split("_")[1] ?? "0"))) {
                                        setCurrentModuleLevel(parseInt(value.split("_")[1] ?? "0"));
                                    }
                                }}
                                defaultValue={currentModuleLevel !== 0 ? `${module.id}_${currentModuleLevel}` : `${module.id}_${moduleData?.find((module) => module.id === currentModule)?.phases?.[(moduleData?.find((module) => module.id === currentModule)?.phases?.length ?? 0) - 1]?.equipLevel ?? 0}`}
                            >
                                <SelectTrigger className="md:w-[180px]">
                                    <SelectValue placeholder="Select a Module Level" />
                                </SelectTrigger>
                                <SelectContent>
                                    {moduleData
                                        ?.find((module) => module.id === currentModule)
                                        ?.phases.map((phase, index) => (
                                            <SelectItem value={`${module.id}_${phase.equipLevel}`} key={index}>
                                                Level {phase.equipLevel}
                                            </SelectItem>
                                        ))}
                                </SelectContent>
                            </Select>
                        ) : (
                            <></>
                        )}
                    </div>
                    <div>
                        <Select
                            defaultValue={"potential_0"}
                            onValueChange={(value) => {
                                setPotentialRank(parseInt(value.split("_")[1] ?? "0"));
                            }}
                        >
                            <SelectTrigger className="md:w-[180px]">
                                <SelectValue placeholder="Potential Rank" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value={"potential_0"}>No Potential</SelectItem>
                                {operator.potentialRanks.map((rank, index) => (
                                    <SelectItem value={`potential_${index + 1}`} key={index}>
                                        Potential {index + 1} - {rank.description}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                </div>
                <div className="max-w-md">
                    <Slider min={0} max={100} step={1} value={[favorPoint]} onValueChange={(value) => setFavorPoint(value[0] ?? 0)} />
                    <div className="mt-2 flex flex-row items-center gap-2 text-sm">
                        <Input type="number" min={0} max={200} value={favorPoint * 2} onChange={(e) => handleFavorPointChange(Number(e.target.value) / 2)} className="w-20" />
                        <span className="text-muted-foreground">Trust</span>
                    </div>
                </div>
            </div>
        </>
    );
};
