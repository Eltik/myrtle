import { ArrowRight, BookOpen } from "lucide-react";
import { Card, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";

export function Guides() {
    return (
        <Card className="group cursor-pointer border-border bg-card">
            <CardHeader>
                <div className="flex items-center justify-between">
                    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                        <BookOpen className="h-5 w-5 text-primary" />
                    </div>
                    <ArrowRight className="h-5 w-5 text-muted-foreground transition-colors group-hover:text-primary" />
                </div>
                <CardTitle className="text-card-foreground text-lg">Guides & Tutorials</CardTitle>
                <CardDescription>Learn strategies for tough stages and optimize your team compositions.</CardDescription>
            </CardHeader>
        </Card>
    );
}
