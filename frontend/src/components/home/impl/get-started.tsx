import { Sparkles, Star, Users } from "lucide-react";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";

export function GetStarted() {
    return (
        <Card className="border-border bg-card md:col-span-2">
            <CardHeader>
                <CardTitle className="text-card-foreground text-xl">Get Started</CardTitle>
                <CardDescription>Create an account to get started with your Arknights journey.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
                <div className="flex items-start gap-4">
                    <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-muted font-medium text-muted-foreground">1</div>
                    <div className="space-y-2">
                        <h3 className="font-semibold text-card-foreground">Connect Your Account</h3>
                        <p className="text-muted-foreground text-sm">Link your Arknights account to sync your operator data and progress.</p>
                        <Button className="gap-2 bg-transparent" size="sm" variant="outline">
                            <Sparkles className="h-4 w-4" />
                            Connect
                        </Button>
                    </div>
                </div>
                <div className="flex items-start gap-4">
                    <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-muted font-medium text-muted-foreground">2</div>
                    <div className="space-y-2">
                        <h3 className="font-semibold text-card-foreground">View Your Profile</h3>
                        <p className="text-muted-foreground text-sm">Access your personalized dashboard with all your operators and stats.</p>
                        <Button className="gap-2 bg-transparent" size="sm" variant="outline">
                            <Users className="h-4 w-4" />
                            Profile
                        </Button>
                    </div>
                </div>
                <div className="flex items-start gap-4">
                    <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-muted font-medium text-muted-foreground">3</div>
                    <div className="space-y-2">
                        <h3 className="font-semibold text-card-foreground">Star on GitHub</h3>
                        <p className="text-muted-foreground text-sm">Support the project by giving us a star on GitHub.</p>
                        <Button className="gap-2 bg-transparent" size="sm" variant="outline">
                            <Star className="h-4 w-4" />
                            GitHub
                        </Button>
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}
