import { TrendingUp } from "lucide-react";
import { Line, LineChart, XAxis, YAxis } from "recharts";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { ChartContainer, ChartTooltip, ChartTooltipContent } from "~/components/ui/shadcn/chart";
import { CHART_COLORS } from "./constants";

interface DateDataItem {
    date: string;
    fullDate: string;
    pulls: number;
}

interface PullActivityChartProps {
    dateData: DateDataItem[];
}

export function PullActivityChart({ dateData }: PullActivityChartProps) {
    if (dateData.length === 0) {
        return null;
    }

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
                        <TrendingUp className="h-5 w-5" />
                        Pull Activity Over Time
                    </CardTitle>
                    <CardDescription>Historical daily pull volume from all contributing players</CardDescription>
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
                            <XAxis dataKey="date" interval="preserveStartEnd" stroke="hsl(var(--muted-foreground))" tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 10 }} tickLine={false} />
                            <YAxis stroke="hsl(var(--muted-foreground))" tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 10 }} tickLine={false} width={50} />
                            <ChartTooltip content={<ChartTooltipContent />} cursor={{ stroke: "hsl(var(--muted-foreground))", strokeDasharray: "5 5" }} />
                            <Line activeDot={{ r: 5, fill: CHART_COLORS.activity }} dataKey="pulls" dot={{ fill: CHART_COLORS.activity, strokeWidth: 0, r: 3 }} stroke={CHART_COLORS.activity} strokeWidth={2} type="monotone" />
                        </LineChart>
                    </ChartContainer>
                </CardContent>
            </Card>
        </InView>
    );
}
