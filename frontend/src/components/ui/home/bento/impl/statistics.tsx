import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { StatsChart } from "./stats-chart";

export function Statistics() {
    return (
        <Card className="bg-card border-border">
            <CardHeader>
                <CardTitle className="text-xl text-card-foreground">Statistics</CardTitle>
                <CardDescription>View our overall website statistics and data.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
                <StatsChart />
                <div className="grid grid-cols-3 gap-4 pt-4 border-t border-border">
                    <div className="text-center">
                        <p className="text-2xl font-bold text-card-foreground">1,224</p>
                        <p className="text-xs text-muted-foreground">Total Visitors</p>
                    </div>
                    <div className="text-center">
                        <p className="text-2xl font-bold text-green-500">+12%</p>
                        <p className="text-xs text-muted-foreground">Growth</p>
                    </div>
                    <div className="text-center">
                        <p className="text-2xl font-bold text-card-foreground">847</p>
                        <p className="text-xs text-muted-foreground">Active Users</p>
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}
