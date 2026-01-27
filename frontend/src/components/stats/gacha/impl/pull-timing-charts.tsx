import { Calendar, Clock } from "lucide-react";
import { Bar, BarChart, XAxis, YAxis } from "recharts";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { ChartContainer, ChartTooltip, ChartTooltipContent } from "~/components/ui/shadcn/chart";
import { CHART_COLORS } from "./constants";

interface HourlyDataItem {
    hour: string;
    pulls: number;
    percentage: number;
}

interface DailyDataItem {
    day: string;
    pulls: number;
    percentage: number;
}

interface PullTimingChartsProps {
    hourlyData: HourlyDataItem[];
    dailyData: DailyDataItem[];
}

export function PullTimingCharts({ hourlyData, dailyData }: PullTimingChartsProps) {
    if (hourlyData.length === 0 && dailyData.length === 0) {
        return null;
    }

    return (
        <div className="grid gap-4 *:min-w-0 md:grid-cols-2">
            {/* Hourly Distribution */}
            <InView
                once
                transition={{ duration: 0.5, ease: "easeOut", delay: 0.1 }}
                variants={{
                    hidden: { opacity: 0, x: -20 },
                    visible: { opacity: 1, x: 0 },
                }}
            >
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <Clock className="h-5 w-5" />
                            Pulls by Hour
                        </CardTitle>
                        <CardDescription>When players are most active</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <ChartContainer
                            className="h-64 w-full"
                            config={{
                                pulls: {
                                    label: "Pulls",
                                    color: "hsl(217, 91%, 60%)",
                                },
                            }}
                        >
                            <BarChart data={hourlyData}>
                                <XAxis dataKey="hour" interval={2} stroke="hsl(var(--muted-foreground))" tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 10 }} tickLine={false} />
                                <YAxis stroke="hsl(var(--muted-foreground))" tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 10 }} tickLine={false} width={40} />
                                <ChartTooltip content={<ChartTooltipContent />} cursor={false} />
                                <Bar dataKey="pulls" fill={CHART_COLORS.hourly} radius={[4, 4, 0, 0]} />
                            </BarChart>
                        </ChartContainer>
                    </CardContent>
                </Card>
            </InView>

            {/* Daily Distribution */}
            <InView
                once
                transition={{ duration: 0.5, ease: "easeOut", delay: 0.2 }}
                variants={{
                    hidden: { opacity: 0, x: 20 },
                    visible: { opacity: 1, x: 0 },
                }}
            >
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <Calendar className="h-5 w-5" />
                            Pulls by Day
                        </CardTitle>
                        <CardDescription>Weekly pull patterns</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <ChartContainer
                            className="h-64 w-full"
                            config={{
                                pulls: {
                                    label: "Pulls",
                                    color: "hsl(142, 71%, 45%)",
                                },
                            }}
                        >
                            <BarChart data={dailyData}>
                                <XAxis dataKey="day" stroke="hsl(var(--muted-foreground))" tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 11 }} tickLine={false} />
                                <YAxis stroke="hsl(var(--muted-foreground))" tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 10 }} tickLine={false} width={40} />
                                <ChartTooltip content={<ChartTooltipContent />} cursor={false} />
                                <Bar dataKey="pulls" fill={CHART_COLORS.daily} radius={[4, 4, 0, 0]} />
                            </BarChart>
                        </ChartContainer>
                    </CardContent>
                </Card>
            </InView>
        </div>
    );
}
