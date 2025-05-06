import { rarityToNumber } from "~/helper";

import { Button } from "~/components/ui/button";
import { Input } from "~/components/ui/input";
import { Label } from "~/components/ui/label";
import { formatProfession } from "~/helper";
import type { Operator } from "~/types/impl/api/static/operator";
import Image from "next/image";
import { cn } from "~/lib/utils";
import { getRarityBorderColor } from "./helper";

export const TopSection = ({ allOperators, squadSize, handleSquadSizeChange, handleRandomize, isLoading, filteredOperators, excludedOperators, randomizedSquad }: { allOperators: Operator[]; squadSize: number; handleSquadSizeChange: (e: React.ChangeEvent<HTMLInputElement>) => void; handleRandomize: () => void; isLoading: boolean; filteredOperators: Operator[]; excludedOperators: Set<string>; randomizedSquad: Operator[] }) => {
    return (
        <>
            {/* Top Section: Options and Results */}
            <div className="mb-6 grid flex-shrink-0 grid-cols-1 gap-6 md:grid-cols-3">
                {/* Column 1: Options */}
                <div className="space-y-4 md:col-span-1">
                    <h2 className="mb-3 text-xl font-semibold">Options</h2>
                    <div>
                        <Label htmlFor="squadSize">Squad Size</Label>
                        <Input id="squadSize" type="number" value={squadSize} onChange={handleSquadSizeChange} min="1" max={allOperators.length > 0 ? allOperators.length : 12} className="mt-1" />
                    </div>
                    <Button onClick={handleRandomize} disabled={isLoading || filteredOperators.filter((op) => op.id && !excludedOperators.has(op.id)).length < squadSize} className="w-full">
                        Randomize Squad
                    </Button>
                </div>

                {/* Column 2 & 3: Randomized Squad */}
                <div className="md:col-span-2">
                    <h2 className="mb-3 text-xl font-semibold">
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
                                    <div key={op.id} className={cn("relative aspect-square overflow-hidden rounded-md bg-card text-card-foreground shadow-sm", getRarityBorderColor(rarityNum))} title={`${op.name} (${displayProfession})`}>
                                        <Image src={imageUrl} alt={op.name} fill sizes="(max-width: 640px) 20vw, 10vw" className={cn("object-cover")} unoptimized />
                                        <div className="${displayRarityColor} absolute inset-x-0 bottom-0 truncate bg-black/60 px-1 py-0.5 text-center text-[10px] font-medium backdrop-blur-sm">{op.name}</div>
                                    </div>
                                );
                            })}
                        </div>
                    ) : (
                        <p className="italic text-gray-500">Click &quot;Randomize Squad&quot; to generate a squad.</p>
                    )}
                </div>
            </div>
        </>
    );
};
