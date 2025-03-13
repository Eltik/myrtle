/* eslint-disable react-hooks/exhaustive-deps */
import { Activity, BadgeDollarSign, Cake, ChevronDown, CircleGauge, Cross, Diamond, FolderPen, Hourglass, MapPinHouse, Ruler, Shield, ShieldBan, Star, Swords, User, Users } from "lucide-react";
import Image from "next/image";
import { useEffect, useMemo, useState } from "react";
import { LevelSlider } from "~/components/operators/components/info-content/impl/level-slider";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";
import { Separator } from "~/components/ui/separator";
import { Slider } from "~/components/ui/slider";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import { formatGroupId, formatNationId, formatProfession, formatSubProfession, getAvatarById, rarityToNumber } from "~/helper";
import { getOperatorAttributeStats } from "~/helper/getAttributeStats";
import type { Module, ModuleData } from "~/types/impl/api/static/modules";
import { OperatorPosition, type Operator } from "~/types/impl/api/static/operator";
import type { Range } from "~/types/impl/api/static/ranges";
import OperatorRange from "../operator-range";
import { Button } from "~/components/ui/button";
import { Input } from "~/components/ui/input";
import { Badge } from "~/components/ui/badge";
import { AnimatePresence, motion } from "framer-motion";
import { ModuleDetails } from "./impl/module-details";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/collapsible";

// https://aceship.github.io/AN-EN-Tags/akhrchars.html?opname=Projekt_Red
// https://sanitygone.help/operators/gavial-the-invincible#page-content

function InfoContent({ operator }: { operator: Operator }) {
    const [showControls, setShowControls] = useState<boolean>(false); // Show controls for the operator
    const [isDescriptionExpanded, setIsDescriptionExpanded] = useState(false);
    const [isModuleDetailsExpanded, setIsModuleDetailsExpanded] = useState(true);

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
        if (value >= 0 && value <= 100) {
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
                <div className="flex w-full flex-col md:grid md:items-center md:gap-4 lg:grid-cols-[auto,1fr,auto]">
                    <div className="flex flex-row-reverse justify-end">
                        <div className="grid grid-cols-[max-content,minmax(0,1fr)] grid-rows-[max-content,max-content] gap-2 px-3 align-baseline sm:px-5">
                            <div className="col-span-2">
                                <span className="text-2xl font-bold md:text-4xl">{operator.name}</span>
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
                                    <span className="truncate text-sm font-medium">{formatSubProfession(operator.subProfessionId)}</span>
                                </div>
                            </div>
                        </div>
                        <div className="m-0 p-0">
                            <div className="relative mb-3 flex h-[104px] w-[104px] items-center justify-center rounded-md border bg-muted/50 backdrop-blur-lg transition-all duration-150 hover:bg-secondary">
                                <div>
                                    <Image src={getAvatarById(operator.id ?? "")} alt={operator.name} width={160} height={160} loading="lazy" decoding="async" />
                                </div>
                                <div className="absolute -bottom-5 left-0 mb-3 flex w-full justify-center">
                                    <div className="flex flex-row items-center justify-center">{operator.rarity && Array.from({ length: rarityToNumber(operator.rarity) }).map((_, index) => <Star key={index} size={16} fill="#ed9634" strokeWidth={2} stroke="#000000" />)}</div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div className="mt-4 grid h-[max-content] grid-cols-3 gap-2 md:mt-0 md:justify-end">
                        <div className="flex flex-col justify-between p-2 md:p-4">
                            <span className="text-sm text-muted-foreground">Nation</span>
                            <span className="truncate text-sm font-normal md:text-lg">{operator.nationId && String(operator.nationId).length > 0 ? formatNationId(String(operator.nationId)) : "N/A"}</span>
                        </div>
                        <div className="flex flex-col justify-between p-2 md:p-4">
                            <span className="text-sm text-muted-foreground">Faction</span>
                            <span className="truncate text-sm font-normal md:text-lg">{operator.groupId && operator.groupId.length > 0 ? formatGroupId(operator.groupId) : "N/A"}</span>
                        </div>
                        <div className="flex flex-col justify-between p-2 md:p-4">
                            <span className="text-sm text-muted-foreground">Position</span>
                            <span className="text-sm font-normal md:text-lg">{operator.position === OperatorPosition.MELEE ? "Melee" : "Ranged"}</span>
                        </div>
                    </div>
                </div>
                <div className="mt-3 block w-full">
                    {/**
                     * @description Operator's description
                     */}
                    <div className="flex flex-col gap-2">
                        <div className="relative">
                            <AnimatePresence initial={false}>
                                <motion.div initial={{ height: "80px" }} animate={{ height: isDescriptionExpanded ? "auto" : "180px" }} exit={{ height: "180px" }} transition={{ duration: 0.3 }} className="overflow-hidden">
                                    <div className="text-sm md:text-base">
                                        <p>{operator.itemUsage}</p>
                                        <p>{operator.itemDesc}</p>
                                    </div>
                                    {operator.profile && (
                                        <>
                                            <Separator className="mt-2" />
                                            <div className="mt-2 flex w-full flex-col gap-3">
                                                <div className="grid gap-2 md:grid-cols-2">
                                                    <div className="space-y-2">
                                                        <div className="flex items-center space-x-2 overflow-hidden">
                                                            <FolderPen className="h-4 w-4 flex-shrink-0" />
                                                            <span className="flex-shrink-0 text-xs text-muted-foreground md:text-sm">Code Name:</span>
                                                            <span className="truncate text-sm font-medium md:text-base">{operator.profile?.basicInfo.codeName}</span>
                                                        </div>
                                                    </div>
                                                    <div className="space-y-2">
                                                        <div className="flex items-center space-x-2 overflow-hidden">
                                                            <Cake className="h-4 w-4 flex-shrink-0" />
                                                            <span className="flex-shrink-0 text-xs text-muted-foreground md:text-sm">Date of Birth:</span>
                                                            <span className="truncate text-sm font-medium md:text-base">{operator.profile?.basicInfo.dateOfBirth}</span>
                                                        </div>
                                                    </div>
                                                    <div className="space-y-2">
                                                        <div className="flex items-center space-x-2 overflow-hidden">
                                                            <MapPinHouse className="h-4 w-4 flex-shrink-0" />
                                                            <span className="flex-shrink-0 text-xs text-muted-foreground md:text-sm">Place of Birth:</span>
                                                            <span className="truncate text-sm font-medium md:text-base">{operator.profile?.basicInfo.placeOfBirth}</span>
                                                        </div>
                                                    </div>
                                                    <div className="space-y-2">
                                                        <div className="flex items-center space-x-2 overflow-hidden">
                                                            <User className="h-4 w-4 flex-shrink-0" />
                                                            <span className="flex-shrink-0 text-xs text-muted-foreground md:text-sm">Gender:</span>
                                                            <span className="truncate text-sm font-medium md:text-base">{operator.profile?.basicInfo.gender}</span>
                                                        </div>
                                                    </div>
                                                    <div className="space-y-2">
                                                        <div className="flex items-center space-x-2 overflow-hidden">
                                                            <Ruler className="h-4 w-4 flex-shrink-0" />
                                                            <span className="flex-shrink-0 text-xs text-muted-foreground md:text-sm">Height:</span>
                                                            <span className="truncate text-sm font-medium md:text-base">{operator.profile?.basicInfo.height}</span>
                                                        </div>
                                                    </div>
                                                    <div className="space-y-2">
                                                        <div className="flex items-center space-x-2 overflow-hidden">
                                                            <Users className="h-4 w-4 flex-shrink-0" />
                                                            <span className="flex-shrink-0 text-xs text-muted-foreground md:text-sm">Race:</span>
                                                            <span className="truncate text-sm font-medium md:text-base">{operator.profile?.basicInfo.race}</span>
                                                        </div>
                                                    </div>
                                                    <div className="space-y-2">
                                                        <div className="flex items-center space-x-2 overflow-hidden">
                                                            <Activity className="h-4 w-4 flex-shrink-0" />
                                                            <span className="flex-shrink-0 text-xs text-muted-foreground md:text-sm">Combat Exp:</span>
                                                            <span className="truncate text-sm font-medium md:text-base">{operator.profile?.basicInfo.combatExperience}</span>
                                                        </div>
                                                    </div>
                                                </div>
                                                <Separator />
                                                <div>
                                                    <h3 className="mb-2 text-lg font-semibold">Physical Examination</h3>
                                                    <div className="grid gap-2 md:grid-cols-2">
                                                        {Object.entries(operator.profile?.physicalExam ?? {}).map(([key, value]) => (
                                                            <Badge key={key} variant="secondary" className="justify-between overflow-hidden text-sm">
                                                                <span className="truncate font-semibold capitalize">{key.replace(/([A-Z])/g, " $1").trim()}</span>
                                                                <span className="ml-1 truncate font-normal">{value}</span>
                                                            </Badge>
                                                        ))}
                                                    </div>
                                                </div>
                                            </div>
                                        </>
                                    )}
                                </motion.div>
                            </AnimatePresence>
                            {!isDescriptionExpanded && <div className="absolute bottom-0 left-0 right-0 h-16 bg-gradient-to-t from-background to-transparent" />}
                        </div>
                        <span onClick={() => setIsDescriptionExpanded(!isDescriptionExpanded)} className="flex cursor-pointer flex-row items-center self-start text-sm transition-all duration-150 hover:text-muted-foreground">
                            {isDescriptionExpanded ? "Show Less" : "Show More"}
                            <ChevronDown className={`ml-auto h-5 w-5 transition-transform ${isDescriptionExpanded ? "rotate-180" : ""}`} />
                        </span>
                    </div>
                    <div className="mt-4">
                        <Button variant={"outline"} onClick={() => setShowControls(!showControls)} className="flex flex-row items-center">
                            {showControls ? "Hide" : "Show"} Controls
                            <ChevronDown className={`ml-auto transition-transform ${showControls ? "rotate-180" : ""}`} />
                        </Button>
                    </div>
                    <div className={`overflow-hidden transition-all duration-300 ${showControls ? "max-h-[1000px] opacity-100" : "max-h-0 opacity-0"}`}>
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
                        <div className="mt-3 max-w-md">
                            <LevelSlider phaseIndex={phaseIndex} maxLevels={operator.phases.map((phase) => phase.maxLevel)} onLevelChange={handleLevelChange} />
                        </div>
                    </div>
                    <Tabs
                        defaultValue={`phase_${operator.phases.length - 1}`}
                        className="mt-4 w-full"
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
                                    <div className="grid grid-cols-1 gap-2 text-sm sm:grid-cols-2 md:text-base">
                                        <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
                                            <div className="flex items-center">
                                                <Cross size={24} />
                                                <span className="ml-2 text-muted-foreground">Health</span>
                                            </div>
                                            <span className="font-bold">{Math.round(attributeStats?.maxHp ?? 0)}</span>
                                        </div>
                                        <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
                                            <div className="flex items-center">
                                                <Swords size={24} />
                                                <span className="ml-2 text-muted-foreground">ATK</span>
                                            </div>
                                            <span className="font-bold">{Math.round(attributeStats?.atk ?? 0)}</span>
                                        </div>
                                        <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
                                            <div className="flex items-center">
                                                <Shield size={24} />
                                                <span className="ml-2 text-muted-foreground">DEF</span>
                                            </div>
                                            <span className="font-bold">{Math.round(attributeStats?.def ?? 0)}</span>
                                        </div>
                                        <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
                                            <div className="flex items-center">
                                                <CircleGauge size={24} />
                                                <span className="ml-2 text-muted-foreground">ATK Interval</span>
                                            </div>
                                            <span className="font-bold">{attributeStats?.attackSpeed.toFixed(2) ?? 0}</span>
                                        </div>
                                        <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
                                            <div className="flex items-center">
                                                <Diamond size={24} />
                                                <span className="ml-2 text-muted-foreground">RES</span>
                                            </div>
                                            <span className="font-bold">{Math.round(attributeStats?.magicResistance ?? 0)}</span>
                                        </div>
                                        <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
                                            <div className="flex items-center">
                                                <ShieldBan size={24} />
                                                <span className="ml-2 text-muted-foreground">Block</span>
                                            </div>
                                            <span className="font-bold">{Math.round(attributeStats?.blockCnt ?? 0)}</span>
                                        </div>
                                        <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
                                            <div className="flex items-center">
                                                <Hourglass size={24} />
                                                <span className="ml-2 text-muted-foreground">Redeploy</span>
                                            </div>
                                            <span className="font-bold">{Math.round(attributeStats?.respawnTime ?? 0)}</span>
                                        </div>
                                        <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
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
                </div>
            </div>
        </div>
    );
}

export default InfoContent;
