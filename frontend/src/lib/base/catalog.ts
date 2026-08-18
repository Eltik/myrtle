import type { FacilityType, ICatalogRoom, IDraftRoom } from "#/lib/api/base";

export type Catalog = Map<FacilityType, ICatalogRoom>;

export function isPlannable(roomType: string, catalog: Catalog): boolean {
    const def = catalog.get(roomType as FacilityType);
    if (!def) return true;

    return def.phases.some((phase) => phase.max_stationed > 0);
}

export function seatsOf(room: IDraftRoom, catalog: Catalog): number {
    return catalog.get(room.room_type)?.phases[room.level - 1]?.max_stationed ?? 0;
}

export function roomLabel(roomType: string, catalog: Catalog): string {
    return catalog.get(roomType as FacilityType)?.name ?? roomType;
}

export function isProduction(roomType: string): boolean {
    return roomType === "MANUFACTURE" || roomType === "TRADING";
}

export function powerOf(room: IDraftRoom, catalog: Catalog): number {
    return catalog.get(room.room_type)?.phases[room.level - 1]?.electricity ?? 0;
}
