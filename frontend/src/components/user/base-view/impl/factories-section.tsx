import type { ManufactureRoom, RoomSlot } from "~/types/api/impl/user";
import { calculateEfficiency } from "./helpers";
import { RoomCard } from "./room-card";

interface FactoriesSectionProps {
    factories: Record<string, ManufactureRoom> | undefined;
    roomSlots: Record<string, RoomSlot> | undefined;
}

export function FactoriesSection({ factories, roomSlots }: FactoriesSectionProps) {
    return (
        <div>
            <h3 className="mb-4 font-semibold text-lg">Factories</h3>
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {factories &&
                    Object.entries(factories).map(([roomId, factory]) => {
                        const roomSlot = roomSlots?.[roomId];
                        const efficiency = calculateEfficiency(factory.display);

                        return (
                            <RoomCard
                                details={[
                                    { label: "Product", value: factory.formulaId ?? "None" },
                                    { label: "Queue", value: `${factory.remainSolutionCnt ?? 0}/${factory.capacity ?? 0}` },
                                ]}
                                efficiency={efficiency}
                                key={roomId}
                                level={roomSlot?.level ?? 1}
                                title="Factory"
                            />
                        );
                    })}
                {(!factories || Object.keys(factories).length === 0) && <p className="text-muted-foreground text-sm">No factories found.</p>}
            </div>
        </div>
    );
}
