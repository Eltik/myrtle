import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { Badge } from "~/components/ui/shadcn/badge";
import { Progress } from "~/components/ui/shadcn/progress";
import type { User } from "~/types/api/impl/user";

interface BaseViewProps {
    data: User;
}

interface TradingPostData {
    state?: number;
    strategy?: string;
    stockLimit?: number;
    stock?: { gain?: { count?: number } }[];
    display?: { base?: number; buff?: number };
}

interface ManufactureData {
    state?: number;
    formulaId?: string;
    remainSolutionCnt?: number;
    capacity?: number;
    display?: { base?: number; buff?: number };
}

export function BaseView({ data }: BaseViewProps) {
    const tradingPosts = data.building?.rooms?.TRADING as Record<string, TradingPostData> | undefined;
    const factories = data.building?.rooms?.MANUFACTURE as Record<string, ManufactureData> | undefined;
    const roomSlots = data.building?.roomSlots;

    return (
        <ScrollArea className="h-[600px] rounded-md border p-4">
            <div className="space-y-6">
                <Card className="border-yellow-500/30 bg-yellow-500/5">
                    <CardContent className="p-4">
                        <p className="text-muted-foreground text-sm">This section is still a work-in-progress. Base data visualization coming soon!</p>
                    </CardContent>
                </Card>

                {/* Trading Posts */}
                <div>
                    <h3 className="mb-4 font-semibold text-lg">Trading Posts</h3>
                    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                        {tradingPosts &&
                            Object.entries(tradingPosts).map(([roomId, post]) => {
                                const roomSlot = roomSlots?.[roomId];
                                const efficiency = post.display ? post.display.base + post.display.buff : 100;

                                return (
                                    <Card key={roomId}>
                                        <CardHeader className="pb-2">
                                            <CardTitle className="flex items-center justify-between text-sm">
                                                <span>Trading Post</span>
                                                <Badge variant="outline">Lv.{roomSlot?.level ?? 1}</Badge>
                                            </CardTitle>
                                        </CardHeader>
                                        <CardContent className="space-y-2">
                                            <div className="flex items-center justify-between text-sm">
                                                <span className="text-muted-foreground">Strategy</span>
                                                <span className="font-medium">{post.strategy === "O_GOLD" ? "LMD" : "Orundum"}</span>
                                            </div>
                                            <div className="flex items-center justify-between text-sm">
                                                <span className="text-muted-foreground">Stock</span>
                                                <span className="font-medium">
                                                    {post.stock?.length ?? 0}/{post.stockLimit ?? 0}
                                                </span>
                                            </div>
                                            <div className="space-y-1">
                                                <div className="flex items-center justify-between text-sm">
                                                    <span className="text-muted-foreground">Efficiency</span>
                                                    <span className="font-medium">{efficiency}%</span>
                                                </div>
                                                <Progress value={Math.min(efficiency, 200) / 2} className="h-1.5" />
                                            </div>
                                        </CardContent>
                                    </Card>
                                );
                            })}
                        {(!tradingPosts || Object.keys(tradingPosts).length === 0) && <p className="text-muted-foreground text-sm">No trading posts found.</p>}
                    </div>
                </div>

                {/* Factories */}
                <div>
                    <h3 className="mb-4 font-semibold text-lg">Factories</h3>
                    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                        {factories &&
                            Object.entries(factories).map(([roomId, factory]) => {
                                const roomSlot = roomSlots?.[roomId];
                                const efficiency = factory.display ? factory.display.base + factory.display.buff : 100;

                                return (
                                    <Card key={roomId}>
                                        <CardHeader className="pb-2">
                                            <CardTitle className="flex items-center justify-between text-sm">
                                                <span>Factory</span>
                                                <Badge variant="outline">Lv.{roomSlot?.level ?? 1}</Badge>
                                            </CardTitle>
                                        </CardHeader>
                                        <CardContent className="space-y-2">
                                            <div className="flex items-center justify-between text-sm">
                                                <span className="text-muted-foreground">Product</span>
                                                <span className="max-w-[120px] truncate font-medium">{factory.formulaId ?? "None"}</span>
                                            </div>
                                            <div className="flex items-center justify-between text-sm">
                                                <span className="text-muted-foreground">Queue</span>
                                                <span className="font-medium">
                                                    {factory.remainSolutionCnt ?? 0}/{factory.capacity ?? 0}
                                                </span>
                                            </div>
                                            <div className="space-y-1">
                                                <div className="flex items-center justify-between text-sm">
                                                    <span className="text-muted-foreground">Efficiency</span>
                                                    <span className="font-medium">{efficiency}%</span>
                                                </div>
                                                <Progress value={Math.min(efficiency, 200) / 2} className="h-1.5" />
                                            </div>
                                        </CardContent>
                                    </Card>
                                );
                            })}
                        {(!factories || Object.keys(factories).length === 0) && <p className="text-muted-foreground text-sm">No factories found.</p>}
                    </div>
                </div>
            </div>
        </ScrollArea>
    );
}
