import { useMemo, useState } from "react";
import { rarityToNumber, stringToOperatorRarity } from "~/helper";
import type { User } from "~/types/impl/api";
import { OperatorRarity } from "~/types/impl/api/static/operator";
import { Input } from "../ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";
import { Button } from "../ui/button";
import { ArrowDown, ArrowUp } from "lucide-react";
import CharacterCard from "./components/character-card";

function Characters({ data }: { data: User }) {
    const [sortBy, setSortBy] = useState<"level" | "rarity" | "obtained">("level");
    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");
    const [filterRarity, setFilterRarity] = useState<OperatorRarity | "all">("all");
    const [searchTerm, setSearchTerm] = useState("");

    const sortedAndFilteredCharacters = useMemo(() => {
        return Object.values(data.troop.chars)
            .filter((char) => (filterRarity === "all" || char.static?.rarity === filterRarity) && char.static?.name.toLowerCase().includes(searchTerm.toLowerCase()))
            .sort((a, b) => {
                let comparison = 0;
                switch (sortBy) {
                    case "level":
                        if (b.evolvePhase !== a.evolvePhase) {
                            comparison = b.evolvePhase - a.evolvePhase;
                        } else {
                            comparison = b.level - a.level;
                        }
                        break;
                    case "rarity":
                        comparison = rarityToNumber(b.static?.rarity ?? OperatorRarity.sixStar) - rarityToNumber(a.static?.rarity ?? OperatorRarity.sixStar);
                        break;
                    case "obtained":
                        comparison = b.gainTime - a.gainTime;
                        break;
                }
                return sortOrder === "asc" ? -comparison : comparison;
            });
    }, [data.troop.chars, sortBy, sortOrder, filterRarity, searchTerm]);

    const toggleSortOrder = () => {
        setSortOrder((prevOrder) => (prevOrder === "asc" ? "desc" : "asc"));
    };
    
    return (
        <>
            <h2 className="text-2xl font-bold">Characters</h2>
            <div className="flex flex-col space-y-4 md:flex-row md:space-x-4 md:space-y-0">
                <Input placeholder="Search operators..." value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} className="md:w-1/3" />
                <Select value={sortBy} onValueChange={(value: "level" | "rarity" | "obtained") => setSortBy(value)}>
                    <SelectTrigger className="transition-all duration-150 hover:bg-secondary md:w-1/4">
                        <SelectValue placeholder="Sort by" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="level" className="cursor-pointer">
                            Sort by Level
                        </SelectItem>
                        <SelectItem value="rarity" className="cursor-pointer">
                            Sort by Rarity
                        </SelectItem>
                        <SelectItem value="obtained" className="cursor-pointer">
                            Sort by Obtained
                        </SelectItem>
                    </SelectContent>
                </Select>
                <Select value={filterRarity.toString()} onValueChange={(value) => setFilterRarity(value === "all" ? "all" : stringToOperatorRarity(value))}>
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
                <Button onClick={toggleSortOrder} variant="outline" size="icon">
                    {sortOrder === "asc" ? <ArrowUp className="h-4 w-4" /> : <ArrowDown className="h-4 w-4" />}
                </Button>
            </div>
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
                {sortedAndFilteredCharacters.map((char) => (
                    <CharacterCard key={char.charId} data={char} />
                ))}
            </div>
        </>
    )
}

export default Characters;