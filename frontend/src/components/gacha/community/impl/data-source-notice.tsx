import { Card, CardContent } from "~/components/ui/shadcn/card";

export function DataSourceNotice() {
    return (
        <Card className="border-muted-foreground/20 bg-muted/20">
            <CardContent className="pt-6">
                <p className="text-center text-muted-foreground text-sm">Statistics are collected anonymously from users who opt-in to share their gacha data. Rates may vary from individual experiences. Data is updated regularly to reflect the latest community trends.</p>
            </CardContent>
        </Card>
    );
}
