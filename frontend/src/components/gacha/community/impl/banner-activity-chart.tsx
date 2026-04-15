"use client";

import { TrendingUp } from "lucide-react";
import { useMemo } from "react";
import { Line, LineChart, ReferenceArea, XAxis, YAxis } from "recharts";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { ChartContainer, ChartTooltip, ChartTooltipContent } from "~/components/ui/shadcn/chart";
import type { BannerOverlay, DateDataItem } from "./banner-helpers";
import { CHART_COLORS } from "./constants";

interface BannerActivityChartProps {
    dateData: DateDataItem[];
    bannerOverlays: BannerOverlay[];
}

export function BannerActivityChart({ dateData, bannerOverlays }: BannerActivityChartProps) {
    const overlayRegions = useMemo(() => {
        if (dateData.length === 0) return [];
        return bannerOverlays
            .map((overlay) => {
                const startEntry = dateData.find((d) => d.fullDate >= overlay.startDate);
                const endEntries = dateData.filter((d) => d.fullDate <= overlay.endDate);
                const endEntry = endEntries[endEntries.length - 1];
                if (!startEntry || !endEntry) return null;
                return {
                    ...overlay,
                    x1: startEntry.date,
                    x2: endEntry.date,
                };
            })
            .filter((r): r is NonNullable<typeof r> => r !== null);
    }, [bannerOverlays, dateData]);

    if (dateData.length === 0) {
        return null;
    }

    return (
        <InView once transition={{ duration: 0.5, ease: "easeOut" }} variants={{ hidden: { opacity: 0, y: 20 }, visible: { opacity: 1, y: 0 } }}>
            <Card>
                <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                        <TrendingUp className="h-5 w-5" />
                        Pull Activity Over Time
                    </CardTitle>
                    <CardDescription>Historical daily pull volume with limited banner periods highlighted</CardDescription>
                </CardHeader>
                <CardContent>
                    <ChartContainer
                        className="h-72 w-full"
                        config={{
                            pulls: {
                                label: "Pulls",
                                color: CHART_COLORS.activity,
                            },
                        }}
                    >
                        <LineChart data={dateData}>
                            {overlayRegions.map((region) => (
                                <ReferenceArea fill={region.color} fillOpacity={0.08} key={region.gachaPoolId} stroke={region.color} strokeOpacity={0.2} x1={region.x1} x2={region.x2} />
                            ))}
                            <XAxis dataKey="date" interval="preserveStartEnd" stroke="hsl(var(--muted-foreground))" tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 10 }} tickLine={false} />
                            <YAxis stroke="hsl(var(--muted-foreground))" tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 10 }} tickLine={false} width={50} />
                            <ChartTooltip content={<ChartTooltipContent />} cursor={{ stroke: "hsl(var(--muted-foreground))", strokeDasharray: "5 5" }} />
                            <Line activeDot={{ r: 5, fill: CHART_COLORS.activity }} dataKey="pulls" dot={{ fill: CHART_COLORS.activity, strokeWidth: 0, r: 3 }} stroke={CHART_COLORS.activity} strokeWidth={2} type="monotone" />
                        </LineChart>
                    </ChartContainer>

                    {/* Banner legend */}
                    {overlayRegions.length > 0 && (
                        <div className="mt-3 flex flex-wrap gap-x-4 gap-y-1">
                            {overlayRegions.map((region) => (
                                <div className="flex items-center gap-1.5 text-xs" key={region.gachaPoolId}>
                                    <div className="h-2 w-3 rounded-sm opacity-60" style={{ backgroundColor: region.color }} />
                                    <span className="max-w-40 truncate text-muted-foreground">{region.gachaPoolName}</span>
                                </div>
                            ))}
                        </div>
                    )}
                </CardContent>
            </Card>
        </InView>
    );
}
