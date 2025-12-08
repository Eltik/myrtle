import { ArrowRight, Users } from "lucide-react";
import { Card, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";

export function OperatorDatabase() {
    return (
        <Card className="group cursor-pointer border-border bg-card">
            <CardHeader>
                <div className="flex items-center justify-between">
                    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                        <Users className="h-5 w-5 text-primary" />
                    </div>
                    <ArrowRight className="h-5 w-5 text-muted-foreground transition-colors group-hover:text-primary" />
                </div>
                <CardTitle className="text-card-foreground text-lg">Operator Database</CardTitle>
                <CardDescription>Browse all operators with detailed stats, skills, and recommended builds.</CardDescription>
            </CardHeader>
        </Card>
    );
}
