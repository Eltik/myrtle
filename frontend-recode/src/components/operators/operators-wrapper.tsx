import { OperatorRarity, type Operator } from "~/types/impl/api/static/operator";
import { formatProfession, rarityToNumber } from "~/helper";
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
    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("asc");
    const [sortBy, setSortBy] = useState<"name" | "rarity" | "stats">("name");
    const [statsSortBy, setStatsSortBy] = useState<"maxHp" | "atk" | "def" | "magicResistance" | "cost" | "baseAttackTime" | "blockCnt">("maxHp");
    const [filterClasses, setFilterClasses] = useState<OperatorProfession[]>([]);
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

    const isChecked = (value: OperatorProfession): Checked => filterClasses.includes(value);
    const handleCheck = (value: OperatorProfession) => {
        if (filterClasses.includes(value)) {
            setFilterClasses(filterClasses.filter((v) => v !== value));
        } else {
            setFilterClasses([...filterClasses, value]);
        }
    };

    const sortedAndFilteredCharacters = useMemo(() => {
        return Object.values(operators)
            .filter((char) => char.name.toLowerCase().includes(searchTerm.toLowerCase()))
            .filter((char) => filterClasses.length === 0 || filterClasses.includes(char.profession))
            .filter((char) => filterRarity === "all" || char.rarity === filterRarity)
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
    }, [operators, searchTerm, filterClasses, filterRarity, sortBy, sortOrder, statsSortBy]);

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
                                    <div className="flex flex-row gap-4">
                                        <NestedDropdown options={options} onSelect={handleSelect} />
                                        <DropdownMenu>
                                            <DropdownMenuTrigger asChild>
                                                <Button variant="outline" className="w-[200px] justify-between">
                                                    <span className="font-normal">Filter Classes</span>
                                                    <ChevronDown className="ml-2 h-4 w-4 opacity-50" />
                                                </Button>
                                            </DropdownMenuTrigger>
                                            <DropdownMenuContent className="w-[200px]">
                                                <DropdownMenuLabel>Classes</DropdownMenuLabel>
                                                <DropdownMenuSeparator />
                                                {Object.keys(OperatorProfession).map((key) => {
                                                    const value = OperatorProfession[key as keyof typeof OperatorProfession];
                                                    return (
                                                        <DropdownMenuCheckboxItem key={value} checked={isChecked(value)} onCheckedChange={() => handleCheck(value)}>
                                                            {formatProfession(value)}
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
