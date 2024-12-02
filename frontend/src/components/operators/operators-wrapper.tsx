import { OperatorRarity, OperatorSubProfession, type Operator } from "~/types/impl/api/static/operator";
import { capitalize, formatProfession, formatSubProfession, rarityToNumber } from "~/helper";
import { ArrowDownFromLine, ArrowUpFromLine, ChevronDown, Filter, List, Table2 } from "lucide-react";
import { Button } from "../ui/button";
import { useMemo, useState } from "react";
import { Input } from "../ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";
import { NestedDropdown } from "../nested-dropdown";
import { DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "../ui/dropdown-menu";
import type { DropdownMenuCheckboxItemProps } from "@radix-ui/react-dropdown-menu";
import { OperatorProfession } from "~/types/impl/api/static/operator";
import { motion, AnimatePresence } from "framer-motion";
import { OperatorsGrid } from "./operators-grid";
import { Checkbox } from "../ui/checkbox";
import { Label } from "../ui/label";
import { Tooltip, TooltipContent, TooltipTrigger } from "../ui/tooltip";
import { OperatorBirthPlace, OperatorRace } from "~/types/impl/api/static/handbook";

/**
 * @author https://wuwatracker.com/resonator
 * A lot of credit to them! They have an amazing design and I copied almost all of it for this.
 * I will be changing it in the future but for now, it's a good placeholder.
 */

type Checked = DropdownMenuCheckboxItemProps["checked"];

export function OperatorsWrapper({ operators }: { operators: Operator[] }) {
    const [searchTerm, setSearchTerm] = useState("");
    const [viewOption, setViewOption] = useState<"grid" | "portrait">("grid");
    const [showOptions, setShowOptions] = useState(false);

    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");
    const [sortBy, setSortBy] = useState<"name" | "rarity" | "stats">("name");

    const [isModule, setIsModule] = useState(false);
    const [canActivateSkill, setCanActivateSkill] = useState(false);

    const [statsSortBy, setStatsSortBy] = useState<"maxHp" | "atk" | "def" | "magicResistance" | "cost" | "baseAttackTime" | "blockCnt">("maxHp");

    const [filterClasses, setFilterClasses] = useState<OperatorProfession[]>([]);
    const [filterSubClasses, setFilterSubClasses] = useState<OperatorSubProfession[]>([]);

    const [filterSkillChargeTypes, setFilterSkillChargeTypes] = useState<("offensive" | "defensive" | "auto")[]>([]);
    const [filterSkillTypes, setFilterSkillTypes] = useState<("auto" | "manual" | "passive")[]>([]);

    const [filterRarity, setFilterRarity] = useState<OperatorRarity | "all">("all");

    const [filterNation, setFilterNation] = useState<OperatorBirthPlace[]>([]);
    const [filterRace, setFilterRace] = useState<OperatorRace[]>([]);
    const [filterGender, setFilterGender] = useState<("Unknown" | "Female" | "Male" | "Conviction")[]>([]);

    const options = [
        {
            label: "Name",
            value: "name",
            default: true,
        },
        {
            label: "Rarity",
            value: "rarity",
        },
        {
            label: "Stats",
            value: "stats",
            subOptions: [
                {
                    label: "HP",
                    value: "maxHp",
                },
                {
                    label: "ATK",
                    value: "atk",
                },
                {
                    label: "DEF",
                    value: "def",
                },
                {
                    label: "RES",
                    value: "magicResistance",
                },
                {
                    label: "Cost",
                    value: "cost",
                },
                {
                    label: "Attack Speed",
                    value: "baseAttackTime",
                },
                {
                    label: "Block Count",
                    value: "blockCnt",
                },
            ],
        },
    ];

    const handleSelect = (value: string) => {
        if (value === "stats") {
            setSortBy("stats");
        } else if (value === "name" || value === "rarity") {
            setSortBy(value);
            setStatsSortBy("maxHp"); // Reset stats sorting when switching to name or rarity
        } else {
            setSortBy("stats");
            setStatsSortBy(value as "maxHp" | "atk" | "def" | "magicResistance" | "cost" | "baseAttackTime" | "blockCnt");
        }
    };

    const isClassChecked = (value: OperatorProfession): Checked => filterClasses.includes(value);
    const handleClassCheck = (value: OperatorProfession) => {
        if (filterClasses.includes(value)) {
            setFilterClasses(filterClasses.filter((v) => v !== value));
        } else {
            setFilterClasses([...filterClasses, value]);
        }
    };

    const isSubClassChecked = (value: OperatorSubProfession): Checked => filterSubClasses.includes(value);
    const handleSubClassCheck = (value: OperatorSubProfession) => {
        if (filterSubClasses.includes(value)) {
            setFilterSubClasses(filterSubClasses.filter((v) => v !== value));
        } else {
            setFilterSubClasses([...filterSubClasses, value]);
        }
    };

    const isSkillTypeChecked = (value: "auto" | "manual" | "passive"): Checked => filterSkillTypes.includes(value);
    const handleSkillTypeCheck = (value: "auto" | "manual" | "passive") => {
        if (filterSkillTypes.includes(value)) {
            setFilterSkillTypes(filterSkillTypes.filter((v) => v !== value));
        } else {
            setFilterSkillTypes([...filterSkillTypes, value]);
        }
    };

    const isSkillChargeTypeChecked = (value: "offensive" | "defensive" | "auto"): Checked => filterSkillChargeTypes.includes(value);
    const handleSkillChargeTypeCheck = (value: "offensive" | "defensive" | "auto") => {
        if (filterSkillChargeTypes.includes(value)) {
            setFilterSkillChargeTypes(filterSkillChargeTypes.filter((v) => v !== value));
        } else {
            setFilterSkillChargeTypes([...filterSkillChargeTypes, value]);
        }
    };

    const isNationChecked = (value: OperatorBirthPlace): Checked => filterNation.includes(value);
    const handleNationCheck = (value: OperatorBirthPlace) => {
        if (filterNation.includes(value)) {
            setFilterNation(filterNation.filter((v) => v !== value));
        } else {
            setFilterNation([...filterNation, value]);
        }
    };

    const isRaceChecked = (value: OperatorRace): Checked => filterRace.includes(value);
    const handleRaceCheck = (value: OperatorRace) => {
        if (filterRace.includes(value)) {
            setFilterRace(filterRace.filter((v) => v !== value));
        } else {
            setFilterRace([...filterRace, value]);
        }
    };

    const sortedAndFilteredCharacters = useMemo(() => {
        return Object.values(operators)
            .map((char) => {
                if (canActivateSkill) {
                    for (const skill of char.skills) {
                        const hasSkill = skill.static?.levels.some((level) => level.spData.spCost > 0 && level.spData.initSp >= level.spData.spCost);
                        if (hasSkill) {
                            return char;
                        } else {
                            return null;
                        }
                    }
                } else {
                    return char;
                }
            })
            .filter(Boolean)
            .map((char) => {
                return char!;
            })
            .map((char) => {
                if (isModule) {
                    // Add module stats to current stats
                    if (char.modules.length > 0) {
                        let avgAtk = 0;
                        let avgHP = 0;
                        let avgDEF = 0;
                        let avgRES = 0;
                        let avgCost = 0;
                        let avgSpeed = 0;
                        let avgATKSPD = 0;
                        let avgBlockCount = 0;

                        for (const opModule of char.modules) {
                            if (opModule.data) {
                                const phase = opModule.data.phases[opModule.data.phases.length - 1];
                                const statChanges = {
                                    atk: 0,
                                    max_hp: 0,
                                    def: 0,
                                    attack_speed: 0,
                                    magic_resistance: 0,
                                    cost: 0,
                                    respawn_time: 0,
                                    block_cnt: 0,
                                };

                                phase?.attributeBlackboard.forEach((iv) => {
                                    if (!(iv.key in statChanges)) {
                                        throw new Error(`Unknown attribute modified: ${iv.key} with value of ${iv.value}`);
                                    }
                                    // @ts-expect-error - This is fine
                                    statChanges[iv.key] += iv.value;
                                });

                                avgAtk += statChanges.atk;
                                avgHP += statChanges.max_hp;
                                avgDEF += statChanges.def;
                                avgRES += statChanges.magic_resistance;
                                avgCost += statChanges.cost;
                                avgSpeed += statChanges.respawn_time;
                                avgATKSPD += statChanges.attack_speed;
                                avgBlockCount += statChanges.block_cnt;
                            }
                        }

                        avgAtk /= char.modules.length;
                        avgHP /= char.modules.length;
                        avgDEF /= char.modules.length;
                        avgRES /= char.modules.length;
                        avgCost /= char.modules.length;
                        avgSpeed /= char.modules.length;
                        avgATKSPD /= char.modules.length;
                        avgBlockCount /= char.modules.length;

                        const lastPhase = char.phases[char.phases.length - 1];

                        if (lastPhase) {
                            const lastPhaseData = lastPhase.attributesKeyFrames[lastPhase.attributesKeyFrames.length - 1]?.data;
                            if (lastPhaseData) {
                                // Store original stats if not already stored
                                if (!(lastPhaseData as unknown as { originalStats: unknown }).originalStats) {
                                    Object.assign(lastPhaseData, {
                                        originalStats: {
                                            atk: lastPhaseData.atk,
                                            maxHp: lastPhaseData.maxHp,
                                            def: lastPhaseData.def,
                                            magicResistance: lastPhaseData.magicResistance,
                                            cost: lastPhaseData.cost,
                                            baseAttackTime: lastPhaseData.baseAttackTime,
                                            blockCnt: lastPhaseData.blockCnt,
                                            attackSpeed: lastPhaseData.attackSpeed,
                                        },
                                    });
                                }

                                // Apply module stats
                                Object.assign(lastPhaseData, {
                                    atk: (lastPhaseData as unknown as { originalStats: { atk: number } }).originalStats.atk + avgAtk,
                                    maxHp: (lastPhaseData as unknown as { originalStats: { maxHp: number } }).originalStats.maxHp + avgHP,
                                    def: (lastPhaseData as unknown as { originalStats: { def: number } }).originalStats.def + avgDEF,
                                    magicResistance: (lastPhaseData as unknown as { originalStats: { magicResistance: number } }).originalStats.magicResistance + avgRES,
                                    cost: (lastPhaseData as unknown as { originalStats: { cost: number } }).originalStats.cost + avgCost,
                                    baseAttackTime: (lastPhaseData as unknown as { originalStats: { baseAttackTime: number } }).originalStats.baseAttackTime + avgSpeed,
                                    blockCnt: (lastPhaseData as unknown as { originalStats: { blockCnt: number } }).originalStats.blockCnt + avgBlockCount,
                                    attackSpeed: (lastPhaseData as unknown as { originalStats: { attackSpeed: number } }).originalStats.attackSpeed + avgATKSPD,
                                });
                            }
                        }
                    }

                    return char;
                } else {
                    const lastPhase = char.phases[char.phases.length - 1];
                    if (lastPhase) {
                        const lastPhaseData = lastPhase.attributesKeyFrames[lastPhase.attributesKeyFrames.length - 1]?.data;
                        if ((lastPhaseData as unknown as { originalStats: unknown })?.originalStats) {
                            Object.assign(lastPhaseData!, (lastPhaseData as unknown as { originalStats: unknown }).originalStats);
                        }
                    }

                    return char;
                }
            })
            .filter((char) => char.name.toLowerCase().includes(searchTerm.toLowerCase()))
            .filter((char) => filterClasses.length === 0 || filterClasses.includes(char.profession))
            .filter((char) => filterSubClasses.length === 0 || filterSubClasses.includes(char.subProfessionId.toUpperCase() as OperatorSubProfession))
            .filter((char) => filterRarity === "all" || char.rarity === filterRarity)
            .filter((char) => filterNation.length === 0 || (char.profile?.basicInfo.placeOfBirth && String(char.profile?.basicInfo.placeOfBirth) !== "" && filterNation.includes(char.profile?.basicInfo.placeOfBirth)))
            .filter((char) => filterRace.length === 0 || (char.profile?.basicInfo.race && String(char.profile?.basicInfo.race) !== "" && filterRace.includes(char.profile?.basicInfo.race)))
            .filter((char) => (filterGender.includes("Male") ? (char.profile?.basicInfo.gender === "Male]" ? true : false) : filterGender.length === 0 || (char.profile?.basicInfo.gender && String(char.profile?.basicInfo.gender) !== "" && filterGender.includes(char.profile?.basicInfo.gender as "Unknown" | "Female" | "Male" | "Conviction"))))
            .filter(
                (char) =>
                    filterSkillTypes.length === 0 ||
                    char.skills.some((skill) =>
                        skill.static?.levels
                            .map((level) => (level.skillType === "AUTO" ? "auto" : level.skillType === "MANUAL" ? "manual" : level.skillType === "PASSIVE" ? "passive" : ""))
                            .filter((v) => v !== "")
                            .filter(Boolean)
                            .some((v) => filterSkillTypes.includes(v)),
                    ),
            )
            .filter(
                (char) =>
                    filterSkillChargeTypes.length === 0 ||
                    char.skills.some((skill) =>
                        skill.static?.levels
                            .map((level) => (level.spData.spType === "INCREASE_WHEN_ATTACK" ? "offensive" : level.spData.spType === "INCREASE_WITH_TIME" ? "auto" : level.spData.spType === "INCREASE_WHEN_TAKEN_DAMAGE" ? "defensive" : ""))
                            .filter((v) => v !== "")
                            .filter(Boolean)
                            .some((v) => filterSkillChargeTypes.includes(v)),
                    ),
            )
            .sort((a, b) => {
                let comparison = 0;
                switch (sortBy) {
                    case "name":
                        comparison = a.name.localeCompare(b.name);
                        break;
                    case "rarity":
                        comparison = rarityToNumber(b.rarity) - rarityToNumber(a.rarity);
                        break;
                    case "stats":
                        const bEvolvePhase = b.phases[b.phases.length - 1]?.attributesKeyFrames[(b.phases[b.phases.length - 1]?.attributesKeyFrames.length ?? 0) - 1]?.data;
                        const aEvolvePhase = a.phases[a.phases.length - 1]?.attributesKeyFrames[(a.phases[a.phases.length - 1]?.attributesKeyFrames.length ?? 0) - 1]?.data;

                        comparison = (Number(bEvolvePhase?.[statsSortBy as keyof typeof bEvolvePhase]) ?? 0) - (Number(aEvolvePhase?.[statsSortBy as keyof typeof aEvolvePhase]) ?? 0);
                        break;
                    default:
                        comparison = 0;
                        break;
                }
                return sortOrder === "asc" ? -comparison : comparison;
            });
    }, [operators, canActivateSkill, isModule, searchTerm, filterClasses, filterSubClasses, filterRarity, filterNation, filterRace, filterGender, filterSkillTypes, filterSkillChargeTypes, sortBy, sortOrder, statsSortBy]);

    return (
        <>
            <div className="container flex max-w-screen-xl auto-rows-auto flex-col gap-8 gap-y-6 px-4 py-8 md:grid md:grid-cols-12 md:px-8 xl:px-4">
                <div className="col-span-full">
                    <div className="flex flex-col gap-4 pb-8">
                        <div className="flex w-full flex-col justify-between gap-4 md:flex-row">
                            <div className="flex items-center justify-between gap-3">
                                <h1 className="relative flex scroll-m-20 items-center gap-2 text-xl font-bold tracking-tight sm:justify-start lg:text-2xl xl:text-3xl">Operators</h1>
                                <Button variant="outline" className="flex flex-row" onClick={() => setViewOption(viewOption === "grid" ? "portrait" : "grid")}>
                                    {viewOption === "grid" ? <Table2 className="h-4 w-4" /> : <List className="h-4 w-4" />}
                                </Button>
                            </div>
                            <div className="flex flex-col gap-4">
                                <div className="relative flex gap-2">
                                    <Input className="flex w-full" placeholder="Search by name..." value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} />
                                    <Button variant="secondary" className="absolute right-0 inline-flex items-center justify-center rounded-l-none p-2 px-3" onClick={() => setShowOptions(!showOptions)}>
                                        <Filter className="h-4 w-4" />
                                    </Button>
                                </div>
                            </div>
                        </div>
                        <AnimatePresence>
                            {showOptions && (
                                <motion.div initial={{ height: 0, opacity: 0, y: -5 }} animate={{ height: "auto", opacity: 1, y: 0 }} exit={{ height: 0, opacity: 0, y: 0 }} transition={{ duration: 0.3, ease: "easeInOut" }} className="flex w-full flex-col flex-wrap gap-3 md:gap-4 lg:flex-nowrap xl:flex-row">
                                    <div className="flex flex-row flex-wrap gap-4">
                                        <NestedDropdown options={options} onSelect={handleSelect} />
                                        <DropdownMenu>
                                            <DropdownMenuTrigger asChild>
                                                <Button variant="outline" className="w-[200px] justify-between">
                                                    <span className="mr-2 truncate">{filterClasses.length === 0 ? <span className="font-normal">Filter Classes</span> : filterClasses.map((v) => formatProfession(v)).join(", ")}</span>
                                                    <ChevronDown className="ml-2 h-4 w-4 opacity-50" />
                                                </Button>
                                            </DropdownMenuTrigger>
                                            <DropdownMenuContent className="max-h-64 w-[200px] overflow-y-scroll">
                                                <DropdownMenuLabel>Classes</DropdownMenuLabel>
                                                <DropdownMenuSeparator />
                                                {Object.keys(OperatorProfession).map((key) => {
                                                    const value = OperatorProfession[key as keyof typeof OperatorProfession];
                                                    return (
                                                        <DropdownMenuCheckboxItem key={value} checked={isClassChecked(value)} onCheckedChange={() => handleClassCheck(value)}>
                                                            {formatProfession(value)}
                                                        </DropdownMenuCheckboxItem>
                                                    );
                                                })}
                                            </DropdownMenuContent>
                                        </DropdownMenu>
                                        <DropdownMenu>
                                            <DropdownMenuTrigger asChild>
                                                <Button variant="outline" className="w-[200px] justify-between">
                                                    <span className="mr-2 truncate">{filterSubClasses.length === 0 ? <span className="font-normal">Filter Subclasses</span> : filterSubClasses.map((v) => formatSubProfession(v.toLowerCase())).join(", ")}</span>
                                                    <ChevronDown className="ml-2 h-4 w-4 opacity-50" />
                                                </Button>
                                            </DropdownMenuTrigger>
                                            <DropdownMenuContent className="max-h-64 w-[200px] overflow-y-scroll">
                                                <DropdownMenuLabel>Subclasses</DropdownMenuLabel>
                                                <DropdownMenuSeparator />
                                                {Object.keys(OperatorSubProfession).map((key) => {
                                                    const value = OperatorSubProfession[key as keyof typeof OperatorSubProfession];
                                                    return (
                                                        <DropdownMenuCheckboxItem key={value} checked={isSubClassChecked(value)} onCheckedChange={() => handleSubClassCheck(value)}>
                                                            {formatSubProfession(value.toLowerCase())}
                                                        </DropdownMenuCheckboxItem>
                                                    );
                                                })}
                                            </DropdownMenuContent>
                                        </DropdownMenu>
                                        <DropdownMenu>
                                            <DropdownMenuTrigger asChild>
                                                <Button variant="outline" className="w-[200px] justify-between">
                                                    <span className="mr-2 truncate">{filterNation.length === 0 ? <span className="font-normal">Filter Place of Birth</span> : filterNation.map((v) => v).join(", ")}</span>
                                                    <ChevronDown className="ml-2 h-4 w-4 opacity-50" />
                                                </Button>
                                            </DropdownMenuTrigger>
                                            <DropdownMenuContent className="max-h-64 w-[200px] overflow-y-scroll">
                                                <DropdownMenuLabel>Nations</DropdownMenuLabel>
                                                <DropdownMenuSeparator />
                                                {Object.keys(OperatorBirthPlace)
                                                    .filter((key) => !isNaN(key as unknown as number))
                                                    .map((key) => OperatorBirthPlace[key as keyof typeof OperatorBirthPlace])
                                                    .sort((a, b) => String(a).localeCompare(String(b)))
                                                    .map((value) => (
                                                        <DropdownMenuCheckboxItem key={value} checked={isNationChecked(value)} onCheckedChange={() => handleNationCheck(value)}>
                                                            {value}
                                                        </DropdownMenuCheckboxItem>
                                                    ))}
                                            </DropdownMenuContent>
                                        </DropdownMenu>
                                        <DropdownMenu>
                                            <DropdownMenuTrigger asChild>
                                                <Button variant="outline" className="w-[200px] justify-between">
                                                    <span className="mr-2 truncate">{filterRace.length === 0 ? <span className="font-normal">Filter by Race</span> : filterRace.map((v) => v).join(", ")}</span>
                                                    <ChevronDown className="ml-2 h-4 w-4 opacity-50" />
                                                </Button>
                                            </DropdownMenuTrigger>
                                            <DropdownMenuContent className="max-h-64 w-[200px] overflow-y-scroll">
                                                <DropdownMenuLabel>Races</DropdownMenuLabel>
                                                <DropdownMenuSeparator />
                                                {Object.keys(OperatorRace)
                                                    .filter((key) => !isNaN(key as unknown as number))
                                                    .map((key) => OperatorRace[key as keyof typeof OperatorRace])
                                                    .sort((a, b) => String(a).localeCompare(String(b)))
                                                    .map((value) => (
                                                        <DropdownMenuCheckboxItem key={value} checked={isRaceChecked(value)} onCheckedChange={() => handleRaceCheck(value)}>
                                                            {value}
                                                        </DropdownMenuCheckboxItem>
                                                    ))}
                                            </DropdownMenuContent>
                                        </DropdownMenu>
                                    </div>
                                    <div className="flex flex-col gap-2">
                                        <div className="flex flex-row flex-wrap gap-4 xl:flex-nowrap">
                                            <Select value={filterRarity} onValueChange={(value) => setFilterRarity(value as OperatorRarity | "all")}>
                                                <SelectTrigger className="w-[200px] transition-all duration-150 hover:bg-secondary">
                                                    <SelectValue placeholder="Filter by Rarity" />
                                                </SelectTrigger>
                                                <SelectContent>
                                                    <SelectItem value="all" className="cursor-pointer">
                                                        All Rarities
                                                    </SelectItem>
                                                    <SelectItem value={OperatorRarity.sixStar} className="cursor-pointer">
                                                        6 Star
                                                    </SelectItem>
                                                    <SelectItem value={OperatorRarity.fiveStar} className="cursor-pointer">
                                                        5 Star
                                                    </SelectItem>
                                                    <SelectItem value={OperatorRarity.fourStar} className="cursor-pointer">
                                                        4 Star
                                                    </SelectItem>
                                                    <SelectItem value={OperatorRarity.threeStar} className="cursor-pointer">
                                                        3 Star
                                                    </SelectItem>
                                                    <SelectItem value={OperatorRarity.twoStar} className="cursor-pointer">
                                                        2 Star
                                                    </SelectItem>
                                                    <SelectItem value={OperatorRarity.oneStar} className="cursor-pointer">
                                                        1 Star
                                                    </SelectItem>
                                                </SelectContent>
                                            </Select>
                                            <DropdownMenu>
                                                <DropdownMenuTrigger asChild>
                                                    <Button variant="outline" className="w-[200px] justify-between">
                                                        <span className="mr-2 truncate">{filterSkillTypes.length === 0 ? <span className="font-normal">Filter Skill Types</span> : filterSkillTypes.map((v) => capitalize(v)).join(", ")}</span>
                                                        <ChevronDown className="ml-2 h-4 w-4 opacity-50" />
                                                    </Button>
                                                </DropdownMenuTrigger>
                                                <DropdownMenuContent className="max-h-64 w-[200px] overflow-y-scroll">
                                                    <DropdownMenuLabel>Skill Types</DropdownMenuLabel>
                                                    <DropdownMenuSeparator />
                                                    <DropdownMenuCheckboxItem checked={isSkillTypeChecked("auto")} onCheckedChange={() => handleSkillTypeCheck("auto")}>
                                                        Auto
                                                    </DropdownMenuCheckboxItem>
                                                    <DropdownMenuCheckboxItem checked={isSkillTypeChecked("manual")} onCheckedChange={() => handleSkillTypeCheck("manual")}>
                                                        Manual
                                                    </DropdownMenuCheckboxItem>
                                                    <DropdownMenuCheckboxItem checked={isSkillTypeChecked("passive")} onCheckedChange={() => handleSkillTypeCheck("passive")}>
                                                        Passive
                                                    </DropdownMenuCheckboxItem>
                                                </DropdownMenuContent>
                                            </DropdownMenu>
                                            <DropdownMenu>
                                                <DropdownMenuTrigger asChild>
                                                    <Button variant="outline" className="w-[200px] justify-between">
                                                        <span className="mr-2 truncate">{filterSkillChargeTypes.length === 0 ? <span className="font-normal">Filter Skill Charge Types</span> : filterSkillChargeTypes.map((v) => capitalize(v)).join(", ")}</span>
                                                        <ChevronDown className="ml-2 h-4 w-4 opacity-50" />
                                                    </Button>
                                                </DropdownMenuTrigger>
                                                <DropdownMenuContent className="max-h-64 w-[200px] overflow-y-scroll">
                                                    <DropdownMenuLabel>Skill Charge Types</DropdownMenuLabel>
                                                    <DropdownMenuSeparator />
                                                    <DropdownMenuCheckboxItem checked={isSkillChargeTypeChecked("offensive")} onCheckedChange={() => handleSkillChargeTypeCheck("offensive")}>
                                                        Offensive
                                                    </DropdownMenuCheckboxItem>
                                                    <DropdownMenuCheckboxItem checked={isSkillChargeTypeChecked("defensive")} onCheckedChange={() => handleSkillChargeTypeCheck("defensive")}>
                                                        Defensive
                                                    </DropdownMenuCheckboxItem>
                                                    <DropdownMenuCheckboxItem checked={isSkillChargeTypeChecked("auto")} onCheckedChange={() => handleSkillChargeTypeCheck("auto")}>
                                                        Auto
                                                    </DropdownMenuCheckboxItem>
                                                </DropdownMenuContent>
                                            </DropdownMenu>
                                        </div>
                                        <div className="flex flex-row gap-4">
                                            <Tooltip>
                                                <TooltipTrigger asChild>
                                                    <div className="flex flex-row">
                                                        <Checkbox checked={isModule} onCheckedChange={() => setIsModule(!isModule)} id="is-module" />
                                                        <Label htmlFor="is-module" className="ml-2">
                                                            Include Modules
                                                        </Label>
                                                    </div>
                                                </TooltipTrigger>
                                                <TooltipContent>Include module stats in operator stats.</TooltipContent>
                                            </Tooltip>
                                            <Tooltip>
                                                <TooltipTrigger asChild>
                                                    <div className="flex flex-row">
                                                        <Checkbox checked={canActivateSkill} onCheckedChange={() => setCanActivateSkill(!canActivateSkill)} id="can-activate-skill" />
                                                        <Label htmlFor="can-activate-skill" className="ml-2">
                                                            100% Skill Activation
                                                        </Label>
                                                    </div>
                                                </TooltipTrigger>
                                                <TooltipContent>Show only operators who have full SP when deployed.</TooltipContent>
                                            </Tooltip>
                                        </div>
                                    </div>
                                    <Button variant="outline" className="flex flex-row" onClick={() => setSortOrder(sortOrder === "asc" ? "desc" : "asc")}>
                                        <span>{sortOrder.toUpperCase()}</span>
                                        {sortOrder === "asc" ? <ArrowUpFromLine className="h-4 w-4" /> : <ArrowDownFromLine className="h-4 w-4" />}
                                    </Button>
                                </motion.div>
                            )}
                        </AnimatePresence>
                    </div>
                </div>
            </div>
            <OperatorsGrid operators={sortedAndFilteredCharacters} />
        </>
    );
}
