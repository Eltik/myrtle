import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { capitalize, rarityToNumber, stringToOperatorRarity } from "~/helper";
import type { User } from "~/types/impl/api";
import { OperatorRarity } from "~/types/impl/api/static/operator";
import { Input } from "../ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";
import { Button } from "../ui/button";
import { ArrowDown, ArrowUp } from "lucide-react";
import CharacterCard from "./components/characters/character-card";

function CharactersGrid({ data }: { data: User }) {
    const [sortBy, setSortBy] = useState<"level" | "rarity" | "obtained">("level");
    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");
    const [filterRarity, setFilterRarity] = useState<OperatorRarity | "all">("all");
    const [searchTerm, setSearchTerm] = useState("");
    const [displayCount, setDisplayCount] = useState(10);

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

    const observer = useRef<IntersectionObserver | null>(null);
    const lastCharacterRef = useCallback(
        (node: HTMLDivElement) => {
            if (observer.current) observer.current.disconnect();
            observer.current = new IntersectionObserver((entries) => {
                if (entries[0]?.isIntersecting && displayCount < sortedAndFilteredCharacters.length) {
                    setDisplayCount((prevCount) => Math.min(prevCount + 40, sortedAndFilteredCharacters.length));
                }
            });
            if (node) observer.current.observe(node);
        },
        [displayCount, sortedAndFilteredCharacters.length],
    );

    useEffect(() => {
        setDisplayCount(40);
    }, [sortBy, sortOrder, filterRarity, searchTerm]);

    return (
        <div className="flex w-full flex-col space-y-6">
            <div className="flex w-full flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div className="flex w-full flex-col gap-4 sm:w-auto sm:flex-row sm:items-center">
                    <Input placeholder="Search operators..." value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} className="w-full sm:w-[300px]" />
                    <Select value={sortBy} onValueChange={(value: "level" | "rarity" | "obtained") => setSortBy(value)}>
                        <SelectTrigger className="w-full sm:w-[200px]">
                            <SelectValue placeholder="Sort by" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="level">Sort by Level</SelectItem>
                            <SelectItem value="rarity">Sort by Rarity</SelectItem>
                            <SelectItem value="obtained">Sort by Obtained</SelectItem>
                        </SelectContent>
                    </Select>
                    <Select value={filterRarity.toString()} onValueChange={(value) => setFilterRarity(value === "all" ? "all" : stringToOperatorRarity(value))}>
                        <SelectTrigger className="w-full sm:w-[200px]">
                            <SelectValue placeholder="Filter by Rarity" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">All Rarities</SelectItem>
                            <SelectItem value="TIER_6">6 Star</SelectItem>
                            <SelectItem value="TIER_5">5 Star</SelectItem>
                            <SelectItem value="TIER_4">4 Star</SelectItem>
                            <SelectItem value="TIER_3">3 Star</SelectItem>
                            <SelectItem value="TIER_2">2 Star</SelectItem>
                            <SelectItem value="TIER_1">1 Star</SelectItem>
                        </SelectContent>
                    </Select>
                    <Button onClick={toggleSortOrder} variant="outline" className="flex w-full items-center justify-center gap-2 sm:w-auto">
                        <span>{capitalize(sortOrder)}</span>
                        {sortOrder === "asc" ? <ArrowUp className="h-4 w-4" /> : <ArrowDown className="h-4 w-4" />}
                    </Button>
                </div>
            </div>

            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                {sortedAndFilteredCharacters.slice(0, displayCount).map((char, index) => (
                    <div key={char.charId} ref={index === displayCount - 1 ? lastCharacterRef : null} className="w-full">
                        <CharacterCard data={char} />
                    </div>
                ))}
            </div>
        </div>
    );
}

export default CharactersGrid;
