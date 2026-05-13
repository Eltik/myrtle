import { useRef, useState } from "react";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { ITierOperator } from "#/lib/api/tier-lists";
import { RARITY_HEX_MUTED } from "#/lib/utils";
import { setOperatorDrag } from "./dnd";
import { useIsDragSource, useStartOperatorDrag } from "./drag-controller";
import styles from "./Editor.module.css";

interface IEditableOpTileProps {
    operator: ITierOperator;
    disabled?: boolean;
    placed?: boolean;
    onActivate?: (operator: ITierOperator) => void;
    onDragStart?: (operatorId: string) => void;
    onDragEnd?: () => void;
    onDragOverChip?: (operatorId: string, side: "before" | "after") => void;
    title?: string;
}

export function EditableOpTile({ operator, disabled, placed, onActivate, onDragStart, onDragEnd, onDragOverChip, title }: IEditableOpTileProps) {
    const color = RARITY_HEX_MUTED[operator.rarity] ?? RARITY_HEX_MUTED[1];
    const isTouchDragging = useIsDragSource(operator.id);
    const startPress = useStartOperatorDrag();
    const [isMouseDragging, setMouseDragging] = useState(false);
    const dragStartedRef = useRef(false);

    const isDragging = isTouchDragging || isMouseDragging;

    return (
        <button
            type="button"
            data-tl-chip-id={operator.id}
            className={styles.opTile}
            style={{ ["--rarity-color" as string]: color }}
            draggable={!disabled}
            data-dragging={isDragging || undefined}
            data-disabled={disabled || undefined}
            aria-label={`${operator.name} (${operator.rarity}★)${placed ? " - already placed" : ""}`}
            title={title ?? `${operator.name} (${operator.rarity}★)`}
            onClick={() => {
                if (dragStartedRef.current) {
                    dragStartedRef.current = false;
                    return;
                }
                if (!disabled) onActivate?.(operator);
            }}
            onPointerDown={(e) => {
                if (disabled) return;
                if (e.pointerType === "touch" || e.pointerType === "pen") {
                    dragStartedRef.current = false;
                    startPress(e, operator.id);
                }
            }}
            onPointerMove={() => {
                if (isTouchDragging) dragStartedRef.current = true;
            }}
            onDragStart={(e) => {
                if (disabled) {
                    e.preventDefault();
                    return;
                }
                setOperatorDrag(e, { operatorId: operator.id });
                setMouseDragging(true);
                onDragStart?.(operator.id);
            }}
            onDragEnd={() => {
                setMouseDragging(false);
                onDragEnd?.();
            }}
            onDragOver={(e) => {
                if (!onDragOverChip) return;
                e.preventDefault();
                const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
                const side = e.clientX - rect.left < rect.width / 2 ? "before" : "after";
                onDragOverChip(operator.id, side);
            }}
        >
            <OperatorAvatar charId={operator.id} name={operator.name} />
            <span className={styles.opRarity} aria-hidden="true" />
        </button>
    );
}
