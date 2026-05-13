import { useState } from "react";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { ITierOperator } from "#/lib/api/tier-lists";
import { RARITY_HEX_MUTED } from "#/lib/utils";
import { setOperatorDrag } from "./dnd";
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
    const [isDragging, setDragging] = useState(false);

    return (
        <button
            type="button"
            className={styles.opTile}
            style={{ ["--rarity-color" as string]: color }}
            draggable={!disabled}
            data-dragging={isDragging || undefined}
            data-disabled={disabled || undefined}
            aria-label={`${operator.name} (${operator.rarity}★)${placed ? " — already placed" : ""}`}
            title={title ?? `${operator.name} (${operator.rarity}★)`}
            onClick={() => !disabled && onActivate?.(operator)}
            onDragStart={(e) => {
                if (disabled) {
                    e.preventDefault();
                    return;
                }
                setOperatorDrag(e, { operatorId: operator.id });
                setDragging(true);
                onDragStart?.(operator.id);
            }}
            onDragEnd={() => {
                setDragging(false);
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
