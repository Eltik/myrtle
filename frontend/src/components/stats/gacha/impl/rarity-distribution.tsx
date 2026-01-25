import { PieChartIcon } from "lucide-react";
import { Cell, Pie, PieChart, ResponsiveContainer } from "recharts";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { ChartContainer, ChartLegend, ChartLegendContent, ChartTooltip, ChartTooltipContent } from "~/components/ui/shadcn/chart";
import { Separator } from "~/components/ui/shadcn/separator";
import type { CollectiveStats } from "~/types/api";
import { type LuckStatus, RARITY_COLORS } from "./constants";

interface RarityDataItem {
    name: string;
    value: number;
    color: string;
}

interface RateComparisonItem {
    rarity: number;
    label: string;
    actual: number;
    expected: number;
    color: string;
    bgColor: string;
}

interface RarityDistributionProps {
    collectiveStats: CollectiveStats;
    rarityData: RarityDataItem[];
    rateComparisonData: readonly RateComparisonItem[];
    luckStatus: LuckStatus;
}

export function RarityDistribution({ collectiveStats, rarityData, rateComparisonData, luckStatus }: RarityDistributionProps) {
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
                    <div className="flex items-center justify-between">
                        <div>
                            <CardTitle className="flex items-center gap-2">
                                <PieChartIcon className="h-5 w-5" />
                                Rarity Distribution
                            </CardTitle>
                            <CardDescription>Breakdown of pulled operators by rarity</CardDescription>
                        </div>
                        {/* Luck Indicator Badge */}
                        <div className={`flex items-center gap-2 rounded-full px-3 py-1.5 ${luckStatus.bg}`}>
                            <luckStatus.icon className={`h-4 w-4 ${luckStatus.color}`} />
                            <span className={`font-semibold text-sm ${luckStatus.color}`}>{luckStatus.label}</span>
                        </div>
                    </div>
                </CardHeader>
                <CardContent>
                    <div className="grid gap-6 lg:grid-cols-2">
                        {/* Pie Chart */}
                        <div>
                            <ChartContainer
                                className="mx-auto aspect-square h-64"
                                config={{
                                    sixStar: {
                                        label: "6-Star",
                                        color: RARITY_COLORS[6].hex,
                                    },
                                    fiveStar: {
                                        label: "5-Star",
                                        color: RARITY_COLORS[5].hex,
                                    },
                                    fourStar: {
                                        label: "4-Star",
                                        color: RARITY_COLORS[4].hex,
                                    },
                                    threeStar: {
                                        label: "3-Star",
                                        color: RARITY_COLORS[3].hex,
                                    },
                                }}
                            >
                                <ResponsiveContainer height="100%" width="100%">
                                    <PieChart>
                                        <ChartTooltip content={<ChartTooltipContent hideLabel />} cursor={false} />
                                        <Pie cx="50%" cy="50%" data={rarityData} dataKey="value" innerRadius={60} nameKey="name" outerRadius={80} paddingAngle={2}>
                                            {rarityData.map((entry) => (
                                                <Cell fill={entry.color} key={`cell-${entry.name}`} />
                                            ))}
                                        </Pie>
                                        <ChartLegend content={<ChartLegendContent />} verticalAlign="bottom" />
                                    </PieChart>
                                </ResponsiveContainer>
                            </ChartContainer>
                        </div>

                        {/* Rate Comparison Bars */}
                        <div className="space-y-4">
                            <h4 className="font-semibold text-sm">Actual vs Expected Rates</h4>
                            {rateComparisonData.map((item) => {
                                const actualPercent = item.actual * 100;
                                const expectedPercent = item.expected * 100;
                                const ratio = item.expected > 0 ? item.actual / item.expected : 0;
                                const isLucky = ratio >= 1;
                                const barWidth = Math.min(ratio * 100, 150); // Cap at 150% for visual

                                return (
                                    <div className="space-y-1.5" key={item.rarity}>
                                        <div className="flex items-center justify-between text-sm">
                                            <span className="font-medium" style={{ color: item.color }}>
                                                {item.label}
                                            </span>
                                            <div className="flex items-center gap-2">
                                                <span className={isLucky ? "text-green-500" : "text-yellow-500"}>{actualPercent.toFixed(2)}%</span>
                                                <span className="text-muted-foreground">/ {expectedPercent.toFixed(0)}%</span>
                                            </div>
                                        </div>
                                        <div className="relative h-2 overflow-hidden rounded-full bg-secondary">
                                            {/* Expected rate marker */}
                                            <div className="absolute top-0 h-full w-0.5 bg-muted-foreground/50" style={{ left: "66.67%" }} />
                                            {/* Actual rate bar */}
                                            <div className={`h-full rounded-full transition-all duration-1000 ${item.bgColor}`} style={{ width: `${barWidth * 0.6667}%` }} />
                                        </div>
                                        <p className="text-muted-foreground text-xs">{isLucky ? <span className="text-green-500">+{((ratio - 1) * 100).toFixed(1)}% above expected</span> : <span className="text-yellow-500">{((1 - ratio) * 100).toFixed(1)}% below expected</span>}</p>
                                    </div>
                                );
                            })}
                        </div>
                    </div>

                    <Separator className="my-6" />

                    {/* Totals Grid */}
                    <div className="grid grid-cols-4 gap-4 text-center">
                        <div>
                            <p className="text-muted-foreground text-sm">6★ Total</p>
                            <p className={`font-bold text-xl ${RARITY_COLORS[6].text}`}>{collectiveStats.totalSixStars.toLocaleString()}</p>
                        </div>
                        <div>
                            <p className="text-muted-foreground text-sm">5★ Total</p>
                            <p className={`font-bold text-xl ${RARITY_COLORS[5].text}`}>{collectiveStats.totalFiveStars.toLocaleString()}</p>
                        </div>
                        <div>
                            <p className="text-muted-foreground text-sm">4★ Total</p>
                            <p className={`font-bold text-xl ${RARITY_COLORS[4].text}`}>{collectiveStats.totalFourStars.toLocaleString()}</p>
                        </div>
                        <div>
                            <p className="text-muted-foreground text-sm">3★ Total</p>
                            <p className={`font-bold text-xl ${RARITY_COLORS[3].text}`}>{collectiveStats.totalThreeStars.toLocaleString()}</p>
                        </div>
                    </div>
                </CardContent>
            </Card>
        </InView>
    );
}
