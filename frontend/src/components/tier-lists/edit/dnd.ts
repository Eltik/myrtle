export const DRAG_MIME = "application/x-tier-entity";

export interface IDragPayload {
    /** The dragged placement's `${kind}:${id}` key. */
    entityKey: string;
}

export function setEntityDrag(e: React.DragEvent, payload: IDragPayload): void {
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData(DRAG_MIME, JSON.stringify(payload));
    e.dataTransfer.setData("text/plain", payload.entityKey);
}

export function readEntityDrag(e: React.DragEvent): IDragPayload | null {
    const raw = e.dataTransfer.getData(DRAG_MIME);
    if (!raw) return null;
    try {
        const parsed = JSON.parse(raw) as IDragPayload;
        if (typeof parsed.entityKey !== "string") return null;
        return parsed;
    } catch {
        return null;
    }
}

export function hasEntityDrag(e: React.DragEvent): boolean {
    return e.dataTransfer.types.includes(DRAG_MIME);
}
