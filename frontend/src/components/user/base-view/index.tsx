import { Card, CardContent } from "~/components/ui/shadcn/card";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import type { ManufactureRoom, TradingRoom, User } from "~/types/api/impl/user";
import { FactoriesSection } from "./impl/factories-section";
import { TradingPostsSection } from "./impl/trading-posts-section";

interface BaseViewProps {
    data: User;
}

export function BaseView({ data }: BaseViewProps) {
    const tradingPosts = data.building?.rooms?.TRADING as Record<string, TradingRoom> | undefined;
    const factories = data.building?.rooms?.MANUFACTURE as Record<string, ManufactureRoom> | undefined;
    const roomSlots = data.building?.roomSlots;

    return (
        <ScrollArea className="h-[600px] rounded-md border p-4">
            <div className="space-y-6">
                <Card className="border-yellow-500/30 bg-yellow-500/5">
                    <CardContent className="p-4">
                        <p className="text-muted-foreground text-sm">This section is still a work-in-progress. Base data visualization coming soon!</p>
                    </CardContent>
                </Card>

                <TradingPostsSection roomSlots={roomSlots} tradingPosts={tradingPosts} />
                <FactoriesSection factories={factories} roomSlots={roomSlots} />
            </div>
        </ScrollArea>
    );
}
