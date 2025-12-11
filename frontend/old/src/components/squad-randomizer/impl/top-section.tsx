import { Loader2 } from "lucide-react";
import Image from "next/image";
import Link from "next/link";
import { useMemo, useState } from "react";
import { useStore } from "zustand";
import { Button } from "~/components/ui/button";
import { Checkbox } from "~/components/ui/checkbox";
import { Input } from "~/components/ui/input";
import { Label } from "~/components/ui/label";
import { formatProfession, rarityToNumber } from "~/helper";
import { cn } from "~/lib/utils";
import { usePlayer } from "~/store";
import type { StoredUser } from "~/types/impl/api";
import type { PlayerResponse } from "~/types/impl/api/impl/player";
import type { Operator } from "~/types/impl/api/static/operator";
import { getRarityBorderColor } from "./helper";

export const TopSection = ({
    allOperators,
    squadSize,
    handleSquadSizeChange,
    handleRandomize,
    isLoading,
    filteredOperators,
    excludedOperators,
    randomizedSquad,
    setExcludedOperators,
}: {
    allOperators: Operator[];
    squadSize: number;
    handleSquadSizeChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
    handleRandomize: () => void;
    isLoading: boolean;
    filteredOperators: Operator[];
    excludedOperators: Set<string>;
    randomizedSquad: Operator[];
    setExcludedOperators: (operators: Set<string>) => void;
}) => {
    const playerData = useStore(usePlayer, (state) => (state as { playerData: StoredUser })?.playerData);
    const [isImporting, setIsImporting] = useState(false);
    const [onlyE1, setOnlyE1] = useState(false);
    const [onlyE2, setOnlyE2] = useState(false);
    const importUserOperators = async () => {
        if (!playerData) return;
        setIsImporting(true);

        try {
            const response = await fetch("/api/player", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    uid: playerData?.status.uid,
                    server: "en",
                }),
            });

            if (!response.ok) {
                console.error("Failed to fetch player data:", response.statusText);
                // TODO: Consider adding user-facing error feedback (e.g., toast notification)
                return;
            }

            const data = (await response.json()) as PlayerResponse;

            if (!data?.[0]?.data?.troop?.chars) {
                console.warn("Player troop data not found or empty in API response.");
                // TODO: Consider adding user-facing feedback
                return;
            }

            // Set of operator IDs the player owns
            const playerOwnedOperatorIds = new Set<string>(
                Object.values(data[0].data.troop.chars)
                    .filter((char) => (onlyE2 ? char.evolvePhase > 1 : onlyE1 ? char.evolvePhase > 0 : char))
                    .map((char) => char.charId),
            );

            // Create a new set based on current exclusions to ensure immutability
            const updatedExcludedOperators = new Set<string>();

            // Add operators the player doesn't own to the exclusion set
            for (const operator of allOperators) {
                if (operator.id?.startsWith("char_") && !playerOwnedOperatorIds.has(operator.id)) {
                    updatedExcludedOperators.add(operator.id);
                }
            }

            setExcludedOperators(updatedExcludedOperators);
        } catch (error) {
            console.error("Error importing user operators:", error);
            // TODO: Consider adding user-facing error feedback
        } finally {
            setIsImporting(false);
        }
    };

    // Memoize the count of available operators for randomization
    const availableOperatorsCount = useMemo(() => {
        return filteredOperators.filter((op) => op.id && !excludedOperators.has(op.id)).length;
    }, [filteredOperators, excludedOperators]);

    return (
        <>
            {/* Top Section: Options and Results */}
            <div className="mb-6 grid flex-shrink-0 grid-cols-1 gap-6 md:grid-cols-3">
                {/* Column 1: Options */}
                <div className="space-y-4 md:col-span-1">
                    <h2 className="mb-3 font-semibold text-xl">Options</h2>
                    <div>
                        <Label htmlFor="squadSize">Squad Size</Label>
                        <Input className="mt-1" id="squadSize" max={allOperators.length > 0 ? allOperators.length : 12} min="1" onChange={handleSquadSizeChange} type="number" value={squadSize} />
                    </div>
                    <Button className="w-full" disabled={isLoading || availableOperatorsCount < squadSize} onClick={handleRandomize}>
                        Randomize Squad
                    </Button>
                    {playerData && (
                        <>
                            <div className="flex items-center space-x-2">
                                <Checkbox checked={onlyE1 || onlyE2} id="onlyE1" onCheckedChange={(checked) => setOnlyE1(checked === "indeterminate" ? false : checked)} />
                                <Label htmlFor="onlyE1">{"> E1"}</Label>
                                <Checkbox checked={onlyE2} id="onlyE2" onCheckedChange={(checked) => setOnlyE2(checked === "indeterminate" ? false : checked)} />
                                <Label htmlFor="onlyE2">Only E2</Label>
                            </div>
                            <Button className="mt-2 w-full" disabled={isImporting} onClick={importUserOperators}>
                                {isImporting ? (
                                    <>
                                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                        Importing...
                                    </>
                                ) : (
                                    "Import My Operators"
                                )}
                            </Button>
                        </>
                    )}
                </div>

                {/* Column 2 & 3: Randomized Squad */}
                <div className="md:col-span-2">
                    <h2 className="mb-3 font-semibold text-xl">
                        Randomized Squad ({randomizedSquad.length} / {squadSize})
                    </h2>
                    {randomizedSquad.length > 0 ? (
                        <div className="grid grid-cols-3 gap-2 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7">
                            {randomizedSquad.map((op) => {
                                if (!op?.id) return null;
                                const displayProfession = formatProfession(op.profession);
                                const imageUrl = `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${op.id}.png`;
                                const rarityNum = rarityToNumber(op.rarity);

                                return (
                                    <Link className={cn("relative aspect-square overflow-hidden rounded-md bg-card text-card-foreground shadow-sm", getRarityBorderColor(rarityNum))} href={`/operators?id=${op.id}`} key={op.id} title={`${op.name} (${displayProfession})`}>
                                        <Image alt={op.name} className={cn("object-cover")} fill sizes="(max-width: 640px) 20vw, 10vw" src={imageUrl} unoptimized />
                                        <div className="absolute inset-x-0 bottom-0 truncate bg-black/60 px-1 py-0.5 text-center font-medium text-[10px] backdrop-blur-sm">{op.name}</div>
                                    </Link>
                                );
                            })}
                        </div>
                    ) : (
                        <p className="text-gray-500 italic">Click &quot;Randomize Squad&quot; to generate a squad.</p>
                    )}
                </div>
            </div>
        </>
    );
};
