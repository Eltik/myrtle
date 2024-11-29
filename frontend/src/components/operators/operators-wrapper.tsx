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

    const [statsSortBy, setStatsSortBy] = useState<"maxHp" | "atk" | "def" | "magicResistance" | "cost" | "baseAttackTime" | "blockCnt">("maxHp");

    const [filterClasses, setFilterClasses] = useState<OperatorProfession[]>([]);
    const [filterSubClasses, setFilterSubClasses] = useState<OperatorSubProfession[]>([]);

    const [filterSkillTypes, setFilterSkillTypes] = useState<("offensive" | "defensive" | "auto")[]>([]);

    const [filterRarity, setFilterRarity] = useState<OperatorRarity | "all">("all");

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

    const isSkillTypeChecked = (value: "offensive" | "defensive" | "auto"): Checked => filterSkillTypes.includes(value);
    const handleSkillTypeCheck = (value: "offensive" | "defensive" | "auto") => {
        if (filterSkillTypes.includes(value)) {
            setFilterSkillTypes(filterSkillTypes.filter((v) => v !== value));
        } else {
            setFilterSkillTypes([...filterSkillTypes, value]);
        }
    };

    const sortedAndFilteredCharacters = useMemo(() => {
        return Object.values(operators)
            .filter((char) => char.name.toLowerCase().includes(searchTerm.toLowerCase()))
            .filter((char) => filterClasses.length === 0 || filterClasses.includes(char.profession))
            .filter((char) => filterSubClasses.length === 0 || filterSubClasses.includes(char.subProfessionId.toUpperCase() as OperatorSubProfession))
            .filter((char) => filterRarity === "all" || char.rarity === filterRarity)
            .filter(
                (char) =>
                    filterSkillTypes.length === 0 ||
                    char.skills.some((skill) =>
                        skill.static?.levels
                            .map((level) => (level.spData.spType === "INCREASE_WHEN_ATTACK" ? "offensive" : level.spData.spType === "INCREASE_WITH_TIME" ? "auto" : level.spData.spType === "INCREASE_WHEN_TAKEN_DAMAGE" ? "defensive" : ""))
                            .filter((v) => v !== "")
                            .filter(Boolean)
                            .some((v) => filterSkillTypes.includes(v)),
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
    }, [operators, searchTerm, filterClasses, filterSubClasses, filterRarity, filterSkillTypes, sortBy, sortOrder, statsSortBy]);

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
                                <motion.div initial={{ height: 0, opacity: 0, y: -5 }} animate={{ height: "auto", opacity: 1, y: 0 }} exit={{ height: 0, opacity: 0, y: 0 }} transition={{ duration: 0.3, ease: "easeInOut" }} className="flex flex-col gap-3 md:flex-row md:gap-4">
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
                                    </div>
                                    <Select value={filterRarity} onValueChange={(value) => setFilterRarity(value as OperatorRarity | "all")}>
                                        <SelectTrigger className="transition-all duration-150 hover:bg-secondary md:w-1/4">
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
                                            <DropdownMenuCheckboxItem checked={isSkillTypeChecked("offensive")} onCheckedChange={() => handleSkillTypeCheck("offensive")}>
                                                Offensive
                                            </DropdownMenuCheckboxItem>
                                            <DropdownMenuCheckboxItem checked={isSkillTypeChecked("defensive")} onCheckedChange={() => handleSkillTypeCheck("defensive")}>
                                                Defensive
                                            </DropdownMenuCheckboxItem>
                                            <DropdownMenuCheckboxItem checked={isSkillTypeChecked("auto")} onCheckedChange={() => handleSkillTypeCheck("auto")}>
                                                Auto
                                            </DropdownMenuCheckboxItem>
                                        </DropdownMenuContent>
                                    </DropdownMenu>
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
