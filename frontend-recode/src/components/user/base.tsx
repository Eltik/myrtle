import type { User } from "~/types/impl/api";
import { ScrollArea } from "../ui/scroll-area";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Progress } from "../ui/progress";
import { Badge } from "../ui/badge";

function Base({ data }: { data: User }) {
    return (
        <>
            <ScrollArea className="h-[100vh] rounded-md border p-4 lg:h-96">
                <Card className="mb-6">
                    <CardHeader>
                        <CardTitle className="text-xl">Base Overview</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div className="space-y-2">
                            <div>
                                <p className="text-sm text-muted-foreground">Drones</p>
                                <Progress value={(data.building.status.labor.value / data.building.status.labor.maxValue) * 100} className="w-64" />
                            </div>
                            <div>
                                <p className="text-sm text-muted-foreground">Trading Posts</p>
                                <div className="flex flex-wrap gap-2">
                                    {Object.values(data.building.rooms.TRADING).map((post, index) => (
                                        <Badge key={index} variant={post.state === 1 ? "default" : "secondary"}>
                                            {post.state === 1 ? "Active" : "Inactive"}
                                        </Badge>
                                    ))}
                                </div>
                            </div>
                            <div>
                                <p className="text-sm text-muted-foreground">Factories</p>
                                <div className="flex flex-wrap gap-2">
                                    {Object.values(data.building.rooms.MANUFACTURE).map((factory, index) => (
                                        <Badge key={index} variant={factory.state === 1 ? "default" : "secondary"}>
                                            {factory.state === 1 ? "Active" : "Inactive"}
                                        </Badge>
                                    ))}
                                </div>
                            </div>
                        </div>
                    </CardContent>
                </Card>
            </ScrollArea>
        </>
    );
}

export default Base;
