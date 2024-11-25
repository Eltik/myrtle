import { BadgeDollarSign, CircleGauge, Cross, Diamond, Hourglass, Shield, ShieldBan, Star, Swords } from "lucide-react";
import Image from "next/image";
import { useEffect, useState } from "react";
import { LevelSlider } from "~/components/operators/components/level-slider";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";
import { Separator } from "~/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import { formatGroupId, formatNationId, formatProfession, formatSubProfession, getAvatarById, rarityToNumber } from "~/helper";
import { getOperatorAttributeStats } from "~/helper/getAttributeStats";
import type { Module, ModuleData } from "~/types/impl/api/static/modules";
import { OperatorPosition, type Operator } from "~/types/impl/api/static/operator";

// https://aceship.github.io/AN-EN-Tags/akhrchars.html?opname=Projekt_Red
// https://sanitygone.help/operators/gavial-the-invincible#page-content

function InfoContent({ operator }: { operator: Operator }) {
    const [attributeStats, setAttributeStats] = useState<Operator["phases"][number]["attributesKeyFrames"][number]["data"] | null>(null); // Attribute stats to display

    const [moduleData, setModuleData] = useState<(ModuleData & { id: string })[] | null>(null); // All modules of the operator
    const [modules, setModules] = useState<Module[]>([]); // Modules of the operator
    const [currentModule, setCurrentModule] = useState<string>(""); // Current module selected to calculate the attribute stats for
    const [currentModuleLevel, setCurrentModuleLevel] = useState<number>(0); // Current module level selected to calculate the attribute stats for

    const [phaseIndex, setPhaseIndex] = useState<number>(operator.phases.length - 1); // Phase index of the operator
    const [level, setLevel] = useState<number>(1); // Level of the operator
    const [favorPoint, setFavorPoint] = useState<number>(0); // Favor point of the operator
    const [potentialRank, setPotentialRank] = useState<number>(0); // Potential rank of the operator

    useEffect(() => {
        void fetchModules();

        setLevel(operator.phases[operator.phases.length - 1]?.maxLevel ?? 1);
    }, [operator]);

    useEffect(() => {
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
    }, [level, phaseIndex, favorPoint, potentialRank, currentModuleLevel, currentModule]);

    async function fetchModules() {
        const data = (await (
            await fetch("/api/static", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    type: "modules",
                    id: operator.id ?? "",
                    method: "charid",
                }),
            })
        ).json()) as {
            modules: Module[];
        };

        setModules(data.modules);

        const promises = [];
        const moduleData: (ModuleData & { id: string })[] = [];
        for (const operatorModule of data.modules) {
            const promise = new Promise<void>((resolve) => {
                fetchModuleData(operatorModule.id ?? "")
                    .then((details) => {
                        moduleData.push({ ...details, id: operatorModule.id ?? "" });
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
        setModuleData(moduleData);

        if (data.modules[data.modules.length - 1]?.id !== undefined) {
            setCurrentModule(data.modules[data.modules.length - 1]!.id!);
            // moduleData?.find((module) => module.id === currentModule)?.phases[moduleData?.find((module) => module.id === currentModule)?.phases.length ?? 0]
            setCurrentModuleLevel(moduleData?.find((module) => module.id === data.modules[data.modules.length - 1]?.id)?.phases[(moduleData?.find((module) => module.id === data.modules[data.modules.length - 1]?.id)?.phases ?? []).length - 1]?.equipLevel ?? 0);
        }
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

    const fetchModuleData = async (moduleId: string): Promise<ModuleData> => {
        const moduleData = (await (
            await fetch("/api/static", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    type: "modules",
                    id: moduleId,
                    method: "details",
                }),
            })
        ).json()) as { details: ModuleData };
        return moduleData.details;
    };

    const handleLevelChange = (newLevel: number) => {
        setLevel(newLevel);
    };

    return (
        <div>
            <div className="p-2 px-4 backdrop-blur-2xl">
                <span className="text-3xl font-bold">Operator Info</span>
            </div>
            <Separator />
            <div className="p-4">
                <div className="grid w-full grid-cols-[max-content,1fr,max-content] items-center">
                    <div className="flex flex-row-reverse justify-end">
                        <div className="grid grid-cols-[max-content,1fr] grid-rows-[max-content,max-content] gap-2 px-5 align-baseline">
                            <div className="col-span-2">
                                <span className="text-4xl font-bold">{operator.name}</span>
                            </div>
                            <div className="flex items-center border">
                                <div className="bg-card p-2">
                                    <div className="max-full box-border inline-block h-8 w-8">
                                        <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/classes/class_${formatProfession(operator.profession).toLowerCase()}.png`} alt={formatProfession(operator.profession)} loading="lazy" width={160} height={160} decoding="async" />
                                    </div>
                                </div>
                                <div className="box-border grid h-full grid-flow-col items-center gap-2 bg-muted p-[8px_12px]">
                                    <div className="max-full box-border inline-block h-8 max-h-8 w-8">
                                        <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/subclass/sub_${operator.subProfessionId}_icon.png`} alt={formatSubProfession(operator.subProfessionId)} loading="lazy" width={160} height={160} decoding="async" />
                                    </div>
                                    <span className="text-sm font-medium">{formatSubProfession(operator.subProfessionId)}</span>
                                </div>
                            </div>
                        </div>
                        <div className="m-0 p-0">
                            <div className="relative mb-3 flex h-[calc(104px)] w-[calc(104px)] items-center justify-center rounded-md border bg-muted/50 backdrop-blur-lg transition-all duration-150 hover:bg-secondary">
                                <div>
                                    <Image src={getAvatarById(operator.id ?? "")} alt={operator.name} width={160} height={160} loading="lazy" decoding="async" />
                                </div>
                                <div className="absolute -bottom-5 left-0 mb-3 flex w-full justify-center">
                                    <div className="flex flex-row items-center justify-center">{operator.rarity && Array.from({ length: rarityToNumber(operator.rarity) }).map((_, index) => <Star key={index} size={16} fill="#ed9634" strokeWidth={2} stroke="#000000" />)}</div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div className="grid h-[max-content] grid-cols-[repeat(3,max-content)] justify-end gap-x-6">
                        <div className="flex flex-col justify-between p-4">
                            <span className="text-sm text-muted-foreground">Nation</span>
                            <span className="text-lg font-normal">{operator.nationId && operator.nationId.length > 0 ? formatNationId(operator.nationId) : "N/A"}</span>
                        </div>
                        <div className="flex flex-col justify-between p-4">
                            <span className="text-sm text-muted-foreground">Faction</span>
                            <span className="text-lg font-normal">{operator.groupId && operator.groupId.length > 0 ? formatGroupId(operator.groupId) : "N/A"}</span>
                        </div>
                        <div className="flex flex-col justify-between p-4">
                            <span className="text-sm text-muted-foreground">Position</span>
                            <span className="text-lg font-normal">{operator.position === OperatorPosition.MELEE ? "Melee" : "Ranged"}</span>
                        </div>
                    </div>
                </div>
                <div className="mt-3 block">
                    <div>
                        <p>{operator.itemUsage}</p>
                        <p>{operator.itemDesc}</p>
                    </div>
                    <div className="mt-3 flex flex-row gap-1">
                        {moduleData && moduleData.length > 0 ? (
                            <Select
                                onValueChange={(value) => {
                                    setCurrentModule(value);
                                    setCurrentModuleLevel(moduleData?.find((module) => module.id === value)?.phases?.[(moduleData?.find((module) => module.id === value)?.phases?.length ?? 0) - 1]?.equipLevel ?? 0);
                                }}
                                defaultValue={modules[modules.length - 1]?.id ?? ""}
                            >
                                <SelectTrigger className="w-[180px]">
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
                        {currentModule.length && moduleData?.find((module) => module.id === currentModule)?.phases !== undefined ? (
                            <Select
                                onValueChange={(value) => {
                                    if (!isNaN(parseInt(value.split("_")[1] ?? "0"))) {
                                        setCurrentModuleLevel(parseInt(value.split("_")[1] ?? "0"));
                                    }
                                }}
                                defaultValue={`${module.id}_${moduleData?.find((module) => module.id === currentModule)?.phases[(moduleData?.find((module) => module.id === currentModule)?.phases.length ?? 0) - 1]?.equipLevel}`}
                            >
                                <SelectTrigger className="w-[180px]">
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
                    <div className="mt-2 max-w-[80%]">
                        <LevelSlider phaseIndex={phaseIndex} maxLevels={operator.phases.map((phase) => phase.maxLevel)} onLevelChange={handleLevelChange} />
                    </div>
                    <Tabs
                        defaultValue={`phase_${operator.phases.length - 1}`}
                        className="mt-6 w-full"
                        onValueChange={(value) => {
                            setPhaseIndex(parseInt(value.split("_")[1] ?? "0"));
                            setLevel(operator.phases[parseInt(value.split("_")[1] ?? "0")]?.maxLevel ?? 1);
                        }}
                    >
                        <div className="rounded-sm bg-muted pt-1">
                            <TabsList>
                                {operator.phases.map((_, index) => (
                                    <TabsTrigger key={index} value={`phase_${index}`} className="data-[state=active]:bg-card/70">
                                        <Image
                                            src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${index}.png`}
                                            width={35}
                                            height={35}
                                            alt="Promotion"
                                            style={{
                                                maxWidth: "100%",
                                                height: "auto",
                                                objectFit: "contain",
                                            }}
                                        />
                                    </TabsTrigger>
                                ))}
                            </TabsList>
                        </div>
                        <div>
                            {operator.phases.map((_, index) => (
                                <TabsContent key={index} value={`phase_${index}`}>
                                    <div className="grid grid-cols-[repeat(2,1fr)] grid-rows-[repeat(5,max-content)] gap-2">
                                        <div className="flex flex-row items-center justify-between bg-muted p-[12px_16px]">
                                            <div className="flex items-center">
                                                <Cross size={24} />
                                                <span className="ml-2 text-muted-foreground">Health</span>
                                            </div>
                                            <span className="font-bold">{Math.round(attributeStats?.maxHp ?? 0)}</span>
                                        </div>
                                        <div className="flex flex-row items-center justify-between bg-muted p-[12px_16px]">
                                            <div className="flex items-center">
                                                <Swords size={24} />
                                                <span className="ml-2 text-muted-foreground">ATK</span>
                                            </div>
                                            <span className="font-bold">{Math.round(attributeStats?.atk ?? 0)}</span>
                                        </div>
                                        <div className="flex flex-row items-center justify-between bg-muted p-[12px_16px]">
                                            <div className="flex items-center">
                                                <Shield size={24} />
                                                <span className="ml-2 text-muted-foreground">DEF</span>
                                            </div>
                                            <span className="font-bold">{Math.round(attributeStats?.def ?? 0)}</span>
                                        </div>
                                        <div className="flex flex-row items-center justify-between bg-muted p-[12px_16px]">
                                            <div className="flex items-center">
                                                <CircleGauge size={24} />
                                                <span className="ml-2 text-muted-foreground">ASPD</span>
                                            </div>
                                            <span className="font-bold">{attributeStats?.attackSpeed.toFixed(2) ?? 0}</span>
                                        </div>
                                        <div className="flex flex-row items-center justify-between bg-muted p-[12px_16px]">
                                            <div className="flex items-center">
                                                <Diamond size={24} />
                                                <span className="ml-2 text-muted-foreground">RES</span>
                                            </div>
                                            <span className="font-bold">{Math.round(attributeStats?.magicResistance ?? 0)}</span>
                                        </div>
                                        <div className="flex flex-row items-center justify-between bg-muted p-[12px_16px]">
                                            <div className="flex items-center">
                                                <ShieldBan size={24} />
                                                <span className="ml-2 text-muted-foreground">Block</span>
                                            </div>
                                            <span className="font-bold">{Math.round(attributeStats?.blockCnt ?? 0)}</span>
                                        </div>
                                        <div className="flex flex-row items-center justify-between bg-muted p-[12px_16px]">
                                            <div className="flex items-center">
                                                <Hourglass size={24} />
                                                <span className="ml-2 text-muted-foreground">Redeploy</span>
                                            </div>
                                            <span className="font-bold">{Math.round(attributeStats?.respawnTime ?? 0)}</span>
                                        </div>
                                        <div className="flex flex-row items-center justify-between bg-muted p-[12px_16px]">
                                            <div className="flex items-center">
                                                <BadgeDollarSign size={24} />
                                                <span className="ml-2 text-muted-foreground">DP Cost</span>
                                            </div>
                                            <span className="font-bold">{Math.round(attributeStats?.cost ?? 0)}</span>
                                        </div>
                                    </div>
                                </TabsContent>
                            ))}
                        </div>
                    </Tabs>
                </div>
            </div>
        </div>
    );
}

export default InfoContent;
