export const DRAG_MIME = "application/x-tier-operator";

export interface IDragPayload {
    operatorId: string;
}

export function setOperatorDrag(e: React.DragEvent, payload: IDragPayload): void {
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData(DRAG_MIME, JSON.stringify(payload));
    e.dataTransfer.setData("text/plain", payload.operatorId);
}

export function readOperatorDrag(e: React.DragEvent): IDragPayload | null {
    const raw = e.dataTransfer.getData(DRAG_MIME);
    if (!raw) return null;
    try {
        const parsed = JSON.parse(raw) as IDragPayload;
        if (typeof parsed.operatorId !== "string") return null;
        return parsed;
    } catch {
        return null;
    }
}

export function hasOperatorDrag(e: React.DragEvent): boolean {
    return e.dataTransfer.types.includes(DRAG_MIME);
}
