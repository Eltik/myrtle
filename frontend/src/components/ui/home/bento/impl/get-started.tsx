import { Button } from "~/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { Sparkles, Star, Users } from "lucide-react";

export function GetStarted() {
    return (
        <Card className="md:col-span-2 bg-card border-border">
            <CardHeader>
                <CardTitle className="text-xl text-card-foreground">Get Started</CardTitle>
                <CardDescription>Create an account to get started with your Arknights journey.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
                <div className="flex items-start gap-4">
                    <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground font-medium">1</div>
                    <div className="space-y-2">
                        <h3 className="font-semibold text-card-foreground">Connect Your Account</h3>
                        <p className="text-sm text-muted-foreground">Link your Arknights account to sync your operator data and progress.</p>
                        <Button variant="outline" size="sm" className="gap-2 bg-transparent">
                            <Sparkles className="h-4 w-4" />
                            Connect
                        </Button>
                    </div>
                </div>
                <div className="flex items-start gap-4">
                    <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground font-medium">2</div>
                    <div className="space-y-2">
                        <h3 className="font-semibold text-card-foreground">View Your Profile</h3>
                        <p className="text-sm text-muted-foreground">Access your personalized dashboard with all your operators and stats.</p>
                        <Button variant="outline" size="sm" className="gap-2 bg-transparent">
                            <Users className="h-4 w-4" />
                            Profile
                        </Button>
                    </div>
                </div>
                <div className="flex items-start gap-4">
                    <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground font-medium">3</div>
                    <div className="space-y-2">
                        <h3 className="font-semibold text-card-foreground">Star on GitHub</h3>
                        <p className="text-sm text-muted-foreground">Support the project by giving us a star on GitHub.</p>
                        <Button variant="outline" size="sm" className="gap-2 bg-transparent">
                            <Star className="h-4 w-4" />
                            GitHub
                        </Button>
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}
