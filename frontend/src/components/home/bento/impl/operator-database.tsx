import { ArrowRight, Users } from "lucide-react";
import { Card, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";

export function OperatorDatabase() {
    return (
        <Card className="card-glow-hover group cursor-pointer border-border bg-card will-change-transform">
            <CardHeader>
                <div className="flex items-center justify-between">
                    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 transition-transform duration-200 ease-out group-hover:scale-110">
                        <Users className="h-5 w-5 text-primary" />
                    </div>
                    <ArrowRight className="h-5 w-5 text-muted-foreground transition-all duration-200 ease-out group-hover:translate-x-1 group-hover:text-primary" />
                </div>
                <CardTitle className="group-hover:-translate-y-0.5 text-card-foreground text-lg transition-transform duration-200 ease-out">Operator Database</CardTitle>
                <CardDescription>Browse all operators with detailed stats, skills, and recommended builds.</CardDescription>
            </CardHeader>
        </Card>
    );
}
