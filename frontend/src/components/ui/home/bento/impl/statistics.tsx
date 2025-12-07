import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { StatsChart } from "./stats-chart";

export function Statistics() {
    return (
        <Card className="bg-card border-border">
            <CardHeader>
                <CardTitle className="text-xl text-card-foreground">Statistics</CardTitle>
                <CardDescription>View our overall website statistics and data.</CardDescription>
            </CardHeader>
            <CardContent>
                <StatsChart />
            </CardContent>
        </Card>
    );
}
