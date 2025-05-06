import type { User } from "~/types/impl/api";
import { ScrollArea } from "../ui/scroll-area";
import TradingPost from "./components/base/trading-post";
import Factory from "./components/base/factory";

function Base({ data }: { data: User }) {
    return (
        <>
            <ScrollArea className="h-[100vh] rounded-md border p-4 lg:h-96">
                <div className="space-y-2">
                    <div className="rounded-md border p-3">
                        <h1 className="text-lg font-bold">This section is still a work-in-progress. Feedback is appreciated!</h1>
                    </div>
                    <div>
                        <p className="text-sm text-muted-foreground">Trading Posts</p>
                        <div className="flex flex-wrap gap-2">
                            {data.building.rooms.TRADING
                                ? Object.values(data.building.rooms.TRADING).map((post, index) => {
                                      const roomSlotKey = Object.keys(data.building.rooms.TRADING)[index]!;
                                      const roomSlot = data.building.roomSlots[roomSlotKey]!;
                                      const roomId = Object.keys(data.building.rooms.TRADING)[index] ?? "";
                                      return <TradingPost data={post} chars={data.building.chars} roomSlot={roomSlot} roomId={roomId} key={index} />;
                                  })
                                : null}
                        </div>
                    </div>
                    <div>
                        <p className="text-sm text-muted-foreground">Factories</p>
                        <div className="flex flex-wrap gap-2">
                            {data.building.rooms.MANUFACTURE
                                ? Object.values(data.building.rooms.MANUFACTURE).map((post, index) => {
                                      const roomSlotKey = Object.keys(data.building.rooms.MANUFACTURE)[index]!;
                                      const roomSlot = data.building.roomSlots[roomSlotKey]!;
                                      const roomId = Object.keys(data.building.rooms.MANUFACTURE)[index] ?? "";
                                      return <Factory data={post} chars={data.building.chars} roomSlot={roomSlot} roomId={roomId} key={index} />;
                                  })
                                : null}
                        </div>
                    </div>
                </div>
            </ScrollArea>
        </>
    );
}

export default Base;
