import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { StatsChart } from "./stats-chart";

export function Statistics() {
    return (
        <Card className="border-border bg-card">
            <CardHeader>
                <CardTitle className="text-card-foreground text-xl">Statistics</CardTitle>
                <CardDescription>View our overall website statistics and data.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
                <StatsChart />
                <div className="grid grid-cols-3 gap-4 border-border border-t pt-4">
                    <div className="text-center">
                        <p className="font-bold text-2xl text-card-foreground">1,224</p>
                        <p className="text-muted-foreground text-xs">Total Visitors</p>
                    </div>
                    <div className="text-center">
                        <p className="font-bold text-2xl text-green-500">+12%</p>
                        <p className="text-muted-foreground text-xs">Growth</p>
                    </div>
                    <div className="text-center">
                        <p className="font-bold text-2xl text-card-foreground">847</p>
                        <p className="text-muted-foreground text-xs">Active Users</p>
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}
