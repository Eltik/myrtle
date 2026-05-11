import { RotateCcw, Users } from "lucide-react";
import type React from "react";
import { Button } from "#/components/ui/button";
import { getAvatarById } from "#/lib/utils";
import type { IRandomizerOperator } from "../types";
import { SlabFrame } from "./SlabFrame";

interface ISquadSlabProps {
    operators: IRandomizerOperator[];
    squadSize: number;
    onReroll: () => void;
}

export function SquadSlab({ operators, squadSize, onReroll }: ISquadSlabProps): React.ReactElement {
    const filled = operators.length;

    return (
        <SlabFrame index="02" kicker="SQUAD" accent="lagoon">
            <div className="flex items-start justify-between gap-3">
                <div className="flex min-w-0 flex-1 items-center gap-2 text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                    <Users aria-hidden="true" className="h-3.5 w-3.5" />
                    <span>
                        Deployment ·{" "}
                        <span className="font-mono normal-case tracking-normal text-foreground/80">
                            {filled}/{squadSize}
                        </span>
                    </span>
                </div>
                <Button onClick={onReroll} size="xs" variant="outline" aria-label="Reroll squad">
                    <RotateCcw aria-hidden="true" />
                    Reroll
                </Button>
            </div>

            <div className="mt-4 grid grid-cols-4 gap-1.5 sm:grid-cols-6 sm:gap-2 md:grid-cols-8 xl:grid-cols-12">
                {operators.map((op, idx) => (
                    <OperatorTile key={`${op.id}-${idx}`} op={op} index={idx} />
                ))}
                {filled === 0 && <p className="col-span-full font-mono text-sm text-muted-foreground">No matching operators — relax your filters.</p>}
            </div>
        </SlabFrame>
    );
}

function OperatorTile({ op, index }: { op: IRandomizerOperator; index: number }) {
    const rarityVar = `var(--rarity-${op.rarity})`;
    const delay = `${Math.min(index, 11) * 28}ms`;
    return (
        <div className="group relative aspect-square overflow-hidden rounded-md border border-border/60 bg-secondary/40 motion-safe:animate-[squadTileIn_.45s_cubic-bezier(.2,.7,.2,1)_both]" style={{ ["--rarity-tint" as string]: rarityVar, animationDelay: delay }}>
            <img
                src={getAvatarById(op.id)}
                alt={op.name}
                loading="lazy"
                decoding="async"
                className="absolute inset-0 h-full w-full object-cover transition-transform duration-300 ease-out group-hover:scale-[1.04]"
                onError={(e) => {
                    e.currentTarget.style.display = "none";
                }}
            />
            <div className="absolute inset-x-0 bottom-0 h-1 bg-(--rarity-tint) mix-blend-multiply dark:mix-blend-screen" />
            <div className="absolute inset-0 flex flex-col justify-end bg-linear-to-t from-black/72 via-transparent to-transparent opacity-0 transition-opacity duration-200 group-hover:opacity-100">
                <p className="truncate px-1.5 pb-1 text-[10.5px] font-medium text-white">{op.name}</p>
            </div>
        </div>
    );
}
