import { OperatorNation, OperatorRarity, type OperatorSubProfession, type Operator } from "~/types/impl/api/static/operator";
import { capitalize, formatProfession, formatSubProfession, rarityToNumber } from "~/helper";
import { ArrowDownFromLine, ArrowUpFromLine, ChevronDown, ChevronLeft, ChevronRight, Filter, List, Table2 } from "lucide-react";
import { Button } from "../ui/button";
import { useMemo, useState } from "react";
import { Input } from "../ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";
import { NestedDropdown } from "../nested-dropdown";
import { DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "../ui/dropdown-menu";
import type { DropdownMenuCheckboxItemProps } from "@radix-ui/react-dropdown-menu";
import { OperatorProfession } from "~/types/impl/api/static/operator";
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
    const [sortBy, setSortBy] = useState<"name" | "rarity" | "limited" | "stats">("name");

    const [isModule, setIsModule] = useState(false);
    const [showLimited, setShowLimited] = useState(true);
    const [canActivateSkill, setCanActivateSkill] = useState(false);

    // Pagination state
    const [currentPage, setCurrentPage] = useState(1);
    const [pageSize, setPageSize] = useState(24);
    const [pageInput, setPageInput] = useState<string>("1");

    const [statsSortBy, setStatsSortBy] = useState<"maxHp" | "atk" | "def" | "magicResistance" | "cost" | "baseAttackTime" | "blockCnt">("maxHp");

    const [filterClasses, setFilterClasses] = useState<OperatorProfession[]>([]);
    const [filterSubClasses, setFilterSubClasses] = useState<OperatorSubProfession[]>([]);

    const [filterSkillChargeTypes, setFilterSkillChargeTypes] = useState<("offensive" | "defensive" | "auto")[]>([]);
    const [filterSkillTypes, setFilterSkillTypes] = useState<("auto" | "manual" | "passive")[]>([]);

    const [filterRarity, setFilterRarity] = useState<OperatorRarity | "all">("all");

    const [filterBirthPlace, setFilterBirthPlace] = useState<OperatorBirthPlace[]>([]);
    const [filterRace, setFilterRace] = useState<OperatorRace[]>([]);
    const [filterNation, setFilterNation] = useState<(keyof OperatorNation)[]>([]);
    const [filterGender, setFilterGender] = useState<("Unknown" | "Female" | "Male" | "Conviction")[]>([]);
    const [filterArtists, setFilterArtists] = useState<string[]>([]);

    // Reset to first page when filters change
    const resetToFirstPage = () => {
        setCurrentPage(1);
    };

    // Add effect to reset page when filters change
    const handleFilterChange = <T extends unknown[]>(callback: (...args: T) => void) => {
        return (...args: T) => {
            callback(...args);
            resetToFirstPage();
        };
    };

    // Override filter setters with versions that reset pagination
    const setFilterClassesWithReset = handleFilterChange(setFilterClasses);
    const setFilterSubClassesWithReset = handleFilterChange(setFilterSubClasses);
    const setFilterRarityWithReset = handleFilterChange(setFilterRarity);
    const setFilterBirthPlaceWithReset = handleFilterChange(setFilterBirthPlace);
    const setFilterRaceWithReset = handleFilterChange(setFilterRace);
    const setFilterNationWithReset = handleFilterChange(setFilterNation);
    const setFilterGenderWithReset = handleFilterChange(setFilterGender);
    const setFilterSkillTypesWithReset = handleFilterChange(setFilterSkillTypes);
    const setFilterSkillChargeTypesWithReset = handleFilterChange(setFilterSkillChargeTypes);
    const setFilterArtistsWithReset = handleFilterChange(setFilterArtists);
    const setSearchTermWithReset = handleFilterChange(setSearchTerm);

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
            label: "Limited",
            value: "limited",
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
        } else if (value === "name" || value === "rarity" || value === "limited") {
            setSortBy(value);
            setStatsSortBy("maxHp"); // Reset stats sorting when switching to name or rarity
        } else {
            setSortBy("stats");
            setStatsSortBy(value as "maxHp" | "atk" | "def" | "magicResistance" | "cost" | "baseAttackTime" | "blockCnt");
        }
        resetToFirstPage();
    };

    // Group operators by profession and sub-profession
    const groupedProfessions = useMemo(() => {
        return operators.reduce(
            (acc, operator) => {
                const profession = operator.profession;
                const subProfessionId = operator.subProfessionId.toUpperCase() as OperatorSubProfession;

                if (!acc[profession]) {
                    acc[profession] = {} as Record<OperatorSubProfession, Operator[]>;
                }
                if (!acc[profession][subProfessionId]) {
                    acc[profession][subProfessionId] = [];
                }

                acc[profession][subProfessionId]?.push(operator);
                return acc;
            },
            {} as Record<OperatorProfession, Record<OperatorSubProfession, Operator[]>>,
        );
    }, [operators]);

    const groupedArtists = useMemo(() => {
        return operators.reduce(
            (acc, operator) => {
                operator.artists.forEach((artist) => {
                    if (!acc[artist]) {
                        acc[artist] = [];
                    }
                    acc[artist].push(operator);
                });
                return acc;
            },
            {} as Record<string, Operator[]>,
        );
    }, [operators]);

    const isClassChecked = (value: OperatorProfession): Checked => filterClasses.includes(value);
    const handleClassCheck = (value: OperatorProfession) => {
        if (filterClasses.includes(value)) {
            setFilterClassesWithReset(filterClasses.filter((v) => v !== value));
        } else {
            setFilterClassesWithReset([...filterClasses, value]);
        }
    };

    const isSubClassChecked = (value: OperatorSubProfession): Checked => filterSubClasses.includes(value);
    const handleSubClassCheck = (value: OperatorSubProfession) => {
        if (filterSubClasses.includes(value)) {
            setFilterSubClassesWithReset(filterSubClasses.filter((v) => v !== value));
        } else {
            setFilterSubClassesWithReset([...filterSubClasses, value]);
        }
    };

    const isSkillTypeChecked = (value: "auto" | "manual" | "passive"): Checked => filterSkillTypes.includes(value);
    const handleSkillTypeCheck = (value: "auto" | "manual" | "passive") => {
        if (filterSkillTypes.includes(value)) {
            setFilterSkillTypesWithReset(filterSkillTypes.filter((v) => v !== value));
        } else {
            setFilterSkillTypesWithReset([...filterSkillTypes, value]);
        }
    };

    const isSkillChargeTypeChecked = (value: "offensive" | "defensive" | "auto"): Checked => filterSkillChargeTypes.includes(value);
    const handleSkillChargeTypeCheck = (value: "offensive" | "defensive" | "auto") => {
        if (filterSkillChargeTypes.includes(value)) {
            setFilterSkillChargeTypesWithReset(filterSkillChargeTypes.filter((v) => v !== value));
        } else {
            setFilterSkillChargeTypesWithReset([...filterSkillChargeTypes, value]);
        }
    };

    const isBirthPlaceChecked = (value: OperatorBirthPlace): Checked => filterBirthPlace.includes(value);
    const handleBirthPlaceCheck = (value: OperatorBirthPlace) => {
        if (filterBirthPlace.includes(value)) {
            setFilterBirthPlaceWithReset(filterBirthPlace.filter((v) => v !== value));
        } else {
            setFilterBirthPlaceWithReset([...filterBirthPlace, value]);
        }
    };

    const isRaceChecked = (value: OperatorRace): Checked => filterRace.includes(value);
    const handleRaceCheck = (value: OperatorRace) => {
        if (filterRace.includes(value)) {
            setFilterRaceWithReset(filterRace.filter((v) => v !== value));
        } else {
            setFilterRaceWithReset([...filterRace, value]);
        }
    };

    const isNationChecked = (value: keyof OperatorNation): Checked => filterNation.includes(value);
    const handleNationChecked = (value: keyof OperatorNation) => {
        if (filterNation.includes(value)) {
            setFilterNationWithReset(filterNation.filter((v) => v !== value));
        } else {
            setFilterNationWithReset([...filterNation, value]);
        }
    };

    const isArtistChecked = (value: string): Checked => filterArtists.includes(value);
    const handleArtistCheck = (value: string) => {
        if (filterArtists.includes(value)) {
            setFilterArtistsWithReset(filterArtists.filter((v) => v !== value));
        } else {
            setFilterArtistsWithReset([...filterArtists, value]);
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
            .map((char) => {
                if (!showLimited) {
                    return char?.handbook?.isLimited ? null : char;
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
            .filter((char) => filterBirthPlace.length === 0 || (char.profile?.basicInfo.placeOfBirth && String(char.profile?.basicInfo.placeOfBirth) !== "" && filterBirthPlace.includes(char.profile?.basicInfo.placeOfBirth)))
            .filter((char) => filterRace.length === 0 || (char.profile?.basicInfo.race && String(char.profile?.basicInfo.race) !== "" && filterRace.includes(char.profile?.basicInfo.race)))
            .filter((char) => filterNation.length === 0 || (char.nationId && String(char.nationId) !== "" && filterNation.includes(char.nationId)))
            .filter((char) => filterGender.length === 0 || (char.profile?.basicInfo.gender && String(char.profile?.basicInfo.gender) !== "" && filterGender.includes(char.profile?.basicInfo.gender as "Unknown" | "Female" | "Male" | "Conviction")))
            .filter((char) => filterArtists.length === 0 || char.artists.some((artist) => filterArtists.includes(artist)))
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
                    case "limited":
                        comparison = (b.handbook?.isLimited ? 1 : 0) - (a.handbook?.isLimited ? 1 : 0);
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
    }, [operators, canActivateSkill, showLimited, isModule, searchTerm, filterClasses, filterSubClasses, filterRarity, filterBirthPlace, filterRace, filterNation, filterGender, filterSkillTypes, filterSkillChargeTypes, sortBy, sortOrder, statsSortBy, filterArtists]);

    // Calculate total number of pages
    const validOperators = sortedAndFilteredCharacters.filter((operator) => operator.id?.startsWith("char"));
    const totalPages = Math.ceil(validOperators.length / pageSize);

    // Handle page navigation
    const goToNextPage = () => {
        if (currentPage < totalPages) {
            setCurrentPage(currentPage + 1);
            setPageInput((currentPage + 1).toString());
        }
    };

    const goToPreviousPage = () => {
        if (currentPage > 1) {
            setCurrentPage(currentPage - 1);
            setPageInput((currentPage - 1).toString());
        }
    };

    // Handle direct page input
    const handlePageInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setPageInput(e.target.value.replace(/[^0-9]/g, ""));
    };

    const handlePageInputSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        navigateToInputPage();
    };

    const navigateToInputPage = () => {
        const pageNumber = parseInt(pageInput);
        if (!isNaN(pageNumber) && pageNumber >= 1 && pageNumber <= totalPages) {
            setCurrentPage(pageNumber);
        } else {
            // Reset to current page if invalid
            setPageInput(currentPage.toString());
        }
    };

    const handlePageInputBlur = () => {
        navigateToInputPage();
    };

    // Handle page size change
    const handlePageSizeChange = (value: string) => {
        setPageSize(Number(value));
        setCurrentPage(1); // Reset to first page when changing page size
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
                                    <Input className="flex w-full" placeholder="Search by name..." value={searchTerm} onChange={(e) => setSearchTermWithReset(e.target.value)} />
                                    <Button variant="secondary" className="absolute right-0 inline-flex items-center justify-center rounded-l-none p-2 px-3" onClick={() => setShowOptions(!showOptions)}>
                                        <Filter className="h-4 w-4" />
                                    </Button>
                                </div>
                            </div>
                        </div>
                        <div className={`${showOptions ? "pointer-events-auto translate-y-0 opacity-100" : "pointer-events-none hidden h-0 -translate-y-5 opacity-0"} flex w-full flex-col flex-wrap gap-3 transition-all duration-150 md:gap-4 lg:flex-nowrap xl:flex-row`}>
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
                                        {Object.keys(OperatorProfession)
                                            .map((key) => OperatorProfession[key as keyof typeof OperatorProfession])
                                            .sort((a, b) => formatProfession(a).localeCompare(formatProfession(b)))
                                            .map((value) => (
                                                <DropdownMenuCheckboxItem key={value} checked={isClassChecked(value)} onCheckedChange={() => handleClassCheck(value)}>
                                                    {formatProfession(value)}
                                                </DropdownMenuCheckboxItem>
                                            ))}
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
                                        {Object.entries(groupedProfessions).map(([profession, subclasses]) => (
                                            <div key={profession}>
                                                <DropdownMenuLabel className="font-semibold">{formatProfession(profession)}</DropdownMenuLabel>
                                                {Object.keys(subclasses).map((subclass) => (
                                                    <DropdownMenuCheckboxItem key={subclass} checked={isSubClassChecked(subclass as OperatorSubProfession)} onCheckedChange={() => handleSubClassCheck(subclass as OperatorSubProfession)}>
                                                        {formatSubProfession(subclass.toLowerCase())}
                                                    </DropdownMenuCheckboxItem>
                                                ))}
                                                <DropdownMenuSeparator />
                                            </div>
                                        ))}
                                    </DropdownMenuContent>
                                </DropdownMenu>
                                <DropdownMenu>
                                    <DropdownMenuTrigger asChild>
                                        <Button variant="outline" className="w-[200px] justify-between">
                                            <span className="mr-2 truncate">{filterBirthPlace.length === 0 ? <span className="font-normal">Filter Place of Birth</span> : filterBirthPlace.map((v) => v).join(", ")}</span>
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
                                                <DropdownMenuCheckboxItem key={value} checked={isBirthPlaceChecked(value)} onCheckedChange={() => handleBirthPlaceCheck(value)}>
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
                                <DropdownMenu>
                                    <DropdownMenuTrigger asChild>
                                        <Button variant="outline" className="w-[200px] justify-between">
                                            <span className="mr-2 truncate">{filterNation.length === 0 ? <span className="font-normal">Filter by Nation</span> : filterNation.map((v) => OperatorNation[v as keyof typeof OperatorNation]).join(", ")}</span>
                                            <ChevronDown className="ml-2 h-4 w-4 opacity-50" />
                                        </Button>
                                    </DropdownMenuTrigger>
                                    <DropdownMenuContent className="max-h-64 w-[200px] overflow-y-scroll">
                                        <DropdownMenuLabel>Nations</DropdownMenuLabel>
                                        <DropdownMenuSeparator />
                                        {Object.keys(OperatorNation)
                                            .sort((a, b) => String(a).localeCompare(String(b)))
                                            .map((key) => {
                                                const value = OperatorNation[key as keyof typeof OperatorNation];
                                                return (
                                                    <DropdownMenuCheckboxItem key={key} checked={isNationChecked(key as keyof OperatorNation)} onCheckedChange={() => handleNationChecked(key as keyof OperatorNation)}>
                                                        {value}
                                                    </DropdownMenuCheckboxItem>
                                                );
                                            })}
                                    </DropdownMenuContent>
                                </DropdownMenu>
                                <DropdownMenu>
                                    <DropdownMenuTrigger asChild>
                                        <Button variant="outline" className="w-[200px] justify-between">
                                            <span className="mr-2 truncate">{filterArtists.length === 0 ? <span className="font-normal">Filter by Artist</span> : filterArtists.join(", ")}</span>
                                            <ChevronDown className="ml-2 h-4 w-4 opacity-50" />
                                        </Button>
                                    </DropdownMenuTrigger>
                                    <DropdownMenuContent className="max-h-64 w-[200px] overflow-y-scroll">
                                        <DropdownMenuLabel>Artists</DropdownMenuLabel>
                                        <DropdownMenuSeparator />
                                        {Object.keys(groupedArtists)
                                            .sort((a, b) => a.localeCompare(b))
                                            .map((artist) => (
                                                <DropdownMenuCheckboxItem key={artist} checked={isArtistChecked(artist)} onCheckedChange={() => handleArtistCheck(artist)}>
                                                    {artist}
                                                </DropdownMenuCheckboxItem>
                                            ))}
                                    </DropdownMenuContent>
                                </DropdownMenu>
                            </div>
                            <div className="flex flex-col gap-2">
                                <div className="flex flex-row flex-wrap gap-4 xl:flex-nowrap">
                                    <Select value={filterRarity} onValueChange={(value) => setFilterRarityWithReset(value as OperatorRarity | "all")}>
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
                                    <Tooltip>
                                        <TooltipTrigger asChild>
                                            <div className="flex flex-row">
                                                <Checkbox checked={showLimited} onCheckedChange={() => setShowLimited(!showLimited)} id="show-limited" />
                                                <Label htmlFor="show-limited" className="ml-2">
                                                    Show Limited
                                                </Label>
                                            </div>
                                        </TooltipTrigger>
                                        <TooltipContent>Whether to show limited operators.</TooltipContent>
                                    </Tooltip>
                                </div>
                            </div>
                            <Button variant="outline" className="flex flex-row" onClick={() => setSortOrder(sortOrder === "asc" ? "desc" : "asc")}>
                                <span>{sortOrder.toUpperCase()}</span>
                                {sortOrder === "asc" ? <ArrowUpFromLine className="h-4 w-4" /> : <ArrowDownFromLine className="h-4 w-4" />}
                            </Button>
                        </div>
                    </div>
                </div>
            </div>
            <div className={`transition-all duration-150 ${showOptions ? "" : "-mt-8"}`}>
                <OperatorsGrid operators={validOperators} currentPage={currentPage} pageSize={pageSize} />

                {/* Pagination Controls */}
                <div className="mt-8 flex items-center justify-center gap-2">
                    <div className="flex flex-col items-center space-y-4 sm:flex-row sm:space-x-6 sm:space-y-0">
                        <div className="flex items-center space-x-2">
                            <span className="text-sm font-medium">Rows per page:</span>
                            <Select value={String(pageSize)} onValueChange={handlePageSizeChange}>
                                <SelectTrigger className="h-8 w-[70px]">
                                    <SelectValue placeholder={pageSize} />
                                </SelectTrigger>
                                <SelectContent side="top">
                                    {[12, 24, 48, 96].map((size) => (
                                        <SelectItem key={size} value={String(size)}>
                                            {size}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>

                        <div className="flex w-[140px] items-center justify-center text-sm font-medium sm:w-[170px]">
                            <form onSubmit={handlePageInputSubmit} className="flex items-center">
                                <span className="mr-1">Page</span>
                                <Input type="tel" inputMode="numeric" pattern="[0-9]*" value={pageInput} onChange={handlePageInputChange} onBlur={handlePageInputBlur} className="h-8 w-12 px-1 text-center sm:w-14" aria-label="Go to page" />
                                <span className="ml-1">of {totalPages}</span>
                            </form>
                        </div>

                        <div className="flex items-center space-x-2">
                            <Button variant="outline" className="h-8 w-8 p-0" onClick={goToPreviousPage} disabled={currentPage === 1}>
                                <span className="sr-only">Go to previous page</span>
                                <ChevronLeft className="h-4 w-4" />
                            </Button>
                            <Button variant="outline" className="h-8 w-8 p-0" onClick={goToNextPage} disabled={currentPage === totalPages}>
                                <span className="sr-only">Go to next page</span>
                                <ChevronRight className="h-4 w-4" />
                            </Button>
                        </div>
                    </div>
                </div>

                {/* Display total results count */}
                <div className="mt-4 text-center text-sm text-muted-foreground">
                    Showing {Math.min((currentPage - 1) * pageSize + 1, validOperators.length)} to {Math.min(currentPage * pageSize, validOperators.length)} of {validOperators.length} operators
                </div>
            </div>
        </>
    );
}
