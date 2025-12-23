import type { RoomSlot, TradingRoom } from "~/types/api/impl/user";
import { calculateEfficiency, formatTradingStrategy } from "./helpers";
import { RoomCard } from "./room-card";

interface TradingPostsSectionProps {
    tradingPosts: Record<string, TradingRoom> | undefined;
    roomSlots: Record<string, RoomSlot> | undefined;
}

export function TradingPostsSection({ tradingPosts, roomSlots }: TradingPostsSectionProps) {
    return (
        <div>
            <h3 className="mb-4 font-semibold text-lg">Trading Posts</h3>
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {tradingPosts &&
                    Object.entries(tradingPosts).map(([roomId, post]) => {
                        const roomSlot = roomSlots?.[roomId];
                        const efficiency = calculateEfficiency(post.display);

                        return (
                            <RoomCard
                                details={[
                                    { label: "Strategy", value: formatTradingStrategy(post.strategy) },
                                    { label: "Stock", value: `${post.stock?.length ?? 0}/${post.stockLimit ?? 0}` },
                                ]}
                                efficiency={efficiency}
                                key={roomId}
                                level={roomSlot?.level ?? 1}
                                title="Trading Post"
                            />
                        );
                    })}
                {(!tradingPosts || Object.keys(tradingPosts).length === 0) && <p className="text-muted-foreground text-sm">No trading posts found.</p>}
            </div>
        </div>
    );
}
