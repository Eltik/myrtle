import { ArrowDownFromLine, ArrowUpFromLine, ChevronDown, Filter, List, Table2 } from "lucide-react";
import { Button } from "../ui/button";
import { useState } from "react";
import { Input } from "../ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";
import { NestedDropdown } from "../nested-dropdown";
import { DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "../ui/dropdown-menu";
import type { DropdownMenuCheckboxItemProps } from "@radix-ui/react-dropdown-menu";
import { OperatorProfession } from "~/types/impl/api/static/operator";
import { formatProfession } from "~/helper";

type Checked = DropdownMenuCheckboxItemProps["checked"];

export function OperatorsHeader() {
    const [viewOption, setViewOption] = useState<"grid" | "portrait">("grid");
    const [showOptions, setShowOptions] = useState(false);
    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("asc");
    const [sortBy, setSortBy] = useState<"name" | "rarity" | "stats">("name");
    const [filterClasses, setFilterClasses] = useState<OperatorProfession[]>([]);

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
                    value: "hp",
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
                    value: "res",
                },
                {
                    label: "Cost",
                    value: "cost",
                },
            ],
        },
    ];

    const handleSelect = (value: string) => {
        setSortBy(value as "name" | "rarity" | "stats");
    };

    const isChecked = (value: OperatorProfession): Checked => filterClasses.includes(value);
    const handleCheck = (value: OperatorProfession) => {
        if (filterClasses.includes(value)) {
            setFilterClasses(filterClasses.filter((v) => v !== value));
        } else {
            setFilterClasses([...filterClasses, value]);
        }
    };

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
                                    <Input className="flex w-full" placeholder="Search by name..." />
                                    <Button variant="secondary" className="absolute right-0 inline-flex items-center justify-center rounded-l-none p-2 px-3" onClick={() => setShowOptions(!showOptions)}>
                                        <Filter className="h-4 w-4" />
                                    </Button>
                                </div>
                            </div>
                        </div>
                        <div className={`flex flex-col gap-3 ${showOptions ? "flex" : "hidden"} md:flex-row md:gap-4`}>
                            <div className="flex flex-row gap-4">
                                <NestedDropdown options={options} onSelect={handleSelect} />
                                <DropdownMenu>
                                    <DropdownMenuTrigger>
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
                            <Select>
                                <SelectTrigger className="transition-all duration-150 hover:bg-secondary md:w-1/4">
                                    <SelectValue placeholder="Filter by Rarity" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="all" className="cursor-pointer">
                                        All Rarities
                                    </SelectItem>
                                    <SelectItem value="TIER_6" className="cursor-pointer">
                                        6 Star
                                    </SelectItem>
                                    <SelectItem value="TIER_5" className="cursor-pointer">
                                        5 Star
                                    </SelectItem>
                                    <SelectItem value="TIER_4" className="cursor-pointer">
                                        4 Star
                                    </SelectItem>
                                    <SelectItem value="TIER_3" className="cursor-pointer">
                                        3 Star
                                    </SelectItem>
                                    <SelectItem value="TIER_2" className="cursor-pointer">
                                        2 Star
                                    </SelectItem>
                                    <SelectItem value="TIER_1" className="cursor-pointer">
                                        1 Star
                                    </SelectItem>
                                </SelectContent>
                            </Select>
                            <Button variant="outline" className="flex flex-row" onClick={() => setSortOrder(sortOrder === "asc" ? "desc" : "asc")}>
                                <span>{sortOrder.toUpperCase()}</span>
                                {sortOrder === "asc" ? <ArrowUpFromLine className="h-4 w-4" /> : <ArrowDownFromLine className="h-4 w-4" />}
                            </Button>
                        </div>
                    </div>
                </div>
            </div>
        </>
    );
}
