import { ArrowRight, Calendar } from "lucide-react";
import { Card, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";

export function EventsTimeline() {
    return (
        <Card className="bg-card border-border group cursor-pointer">
            <CardHeader>
                <div className="flex items-center justify-between">
                    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                        <Calendar className="h-5 w-5 text-primary" />
                    </div>
                    <ArrowRight className="h-5 w-5 text-muted-foreground group-hover:text-primary transition-colors" />
                </div>
                <CardTitle className="text-lg text-card-foreground">Event Timeline</CardTitle>
                <CardDescription>Stay updated with ongoing and upcoming events, banners, and rewards.</CardDescription>
            </CardHeader>
        </Card>
    );
}
