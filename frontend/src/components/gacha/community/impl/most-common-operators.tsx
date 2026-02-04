import { Users } from "lucide-react";
import Image from "next/image";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import type { OperatorPopularity } from "~/types/api";
import { RARITY_COLORS } from "./constants";

interface OperatorsByRarity {
    6: OperatorPopularity[];
    5: OperatorPopularity[];
    4: OperatorPopularity[];
    3: OperatorPopularity[];
}

interface MostCommonOperatorsProps {
    operatorsByRarity: OperatorsByRarity;
}

export function MostCommonOperators({ operatorsByRarity }: MostCommonOperatorsProps) {
    return (
        <InView
            once
            transition={{ duration: 0.5, ease: "easeOut" }}
            variants={{
                hidden: { opacity: 0, y: 20 },
                visible: { opacity: 1, y: 0 },
            }}
        >
            <Card>
                <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                        <Users className="h-5 w-5" />
                        Most Common Operators
                    </CardTitle>
                    <CardDescription>Top operators pulled by the community</CardDescription>
                </CardHeader>
                <CardContent>
                    <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
                        {([6, 5, 4, 3] as const).map((rarity) => {
                            const operators = operatorsByRarity[rarity];
                            const colors = RARITY_COLORS[rarity];

                            return (
                                <div className="space-y-3" key={rarity}>
                                    <div className="flex items-center gap-2">
                                        <span className={`font-semibold text-sm ${colors.text}`}>{rarity}★ Operators</span>
                                    </div>
                                    {operators.length > 0 ? (
                                        <div className="space-y-2">
                                            {operators.slice(0, 5).map((op, index) => (
                                                <TooltipProvider key={op.charId}>
                                                    <Tooltip>
                                                        <TooltipTrigger asChild>
                                                            <div className={`flex cursor-help items-center gap-2 rounded-lg border p-2 transition-colors hover:bg-accent/50 ${colors.bg} ${colors.border}`}>
                                                                <span className="font-semibold text-muted-foreground text-xs">#{index + 1}</span>
                                                                <div className="relative h-8 w-8 shrink-0 overflow-hidden rounded-md border border-border/50">
                                                                    <Image alt={op.charName} className="object-cover" fill sizes="32px" src={`/api/cdn/avatar/${encodeURIComponent(op.charId)}`} />
                                                                </div>
                                                                <div className="min-w-0 flex-1">
                                                                    <p className="truncate font-medium text-xs">{op.charName.length > 0 ? op.charName : op.charId}</p>
                                                                </div>
                                                                <span className={`shrink-0 font-bold text-sm ${colors.text}`}>×{op.pullCount.toLocaleString()}</span>
                                                            </div>
                                                        </TooltipTrigger>
                                                        <TooltipContent>
                                                            <p className="font-semibold">{op.charName}</p>
                                                            <p className="text-xs">
                                                                {op.pullCount.toLocaleString()} pulls ({(op.percentage * 100).toFixed(2)}%)
                                                            </p>
                                                        </TooltipContent>
                                                    </Tooltip>
                                                </TooltipProvider>
                                            ))}
                                        </div>
                                    ) : (
                                        <p className="py-4 text-center text-muted-foreground text-xs">No data</p>
                                    )}
                                </div>
                            );
                        })}
                    </div>
                </CardContent>
            </Card>
        </InView>
    );
}
