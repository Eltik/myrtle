import type { ComponentPropsWithoutRef, Ref } from "react";
import { useRef, useState } from "react";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { ITierOperator } from "#/lib/api/tier-lists";
import { cn, RARITY_HEX_MUTED } from "#/lib/utils";
import { setOperatorDrag } from "./dnd";
import { useIsDragSource, useStartOperatorDrag } from "./drag-controller";
import styles from "./Editor.module.css";

type ButtonExtras = Omit<ComponentPropsWithoutRef<"button">, "title" | "onDragStart" | "onDragEnd" | "onClick" | "onPointerDown" | "onPointerMove" | "onDragOver" | "ref" | "className" | "style">;

interface IEditableOpTileProps extends ButtonExtras {
    operator: ITierOperator;
    disabled?: boolean;
    placed?: boolean;
    onActivate?: (operator: ITierOperator) => void;
    onDragStart?: (operatorId: string) => void;
    onDragEnd?: () => void;
    onDragOverChip?: (operatorId: string, side: "before" | "after") => void;
    title?: string;
    ref?: Ref<HTMLButtonElement>;
    className?: string;
}

export function EditableOpTile({ operator, disabled, placed, onActivate, onDragStart, onDragEnd, onDragOverChip, title, ref, className, ...rest }: IEditableOpTileProps) {
    const color = RARITY_HEX_MUTED[operator.rarity] ?? RARITY_HEX_MUTED[1];
    const isTouchDragging = useIsDragSource(operator.id);
    const startPress = useStartOperatorDrag();
    const [isMouseDragging, setMouseDragging] = useState(false);
    const dragStartedRef = useRef(false);
    // Cache the chip's center-x for the duration of one hover so we don't call
    // getBoundingClientRect on every dragover event, and skip the parent
    // callback unless the resolved side actually changes.
    const centerXRef = useRef<number | null>(null);
    const sideRef = useRef<"before" | "after" | null>(null);

    const isDragging = isTouchDragging || isMouseDragging;

    return (
        <button
            {...rest}
            ref={ref}
            type="button"
            data-tl-chip-id={operator.id}
            className={cn(styles.opTile, className)}
            style={{ ["--rarity-color" as string]: color }}
            draggable={!disabled}
            data-dragging={isDragging || undefined}
            data-disabled={disabled || undefined}
            aria-disabled={disabled || undefined}
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
                e.dataTransfer.clearData();
                setOperatorDrag(e, { operatorId: operator.id });
                setMouseDragging(true);
                onDragStart?.(operator.id);
            }}
            onDragEnd={() => {
                setMouseDragging(false);
                centerXRef.current = null;
                sideRef.current = null;
                onDragEnd?.();
            }}
            onDragOver={(e) => {
                if (!onDragOverChip) return;
                e.preventDefault();
                let centerX = centerXRef.current;
                if (centerX === null) {
                    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
                    centerX = rect.left + rect.width / 2;
                    centerXRef.current = centerX;
                }
                const side: "before" | "after" = e.clientX < centerX ? "before" : "after";
                if (side === sideRef.current) return;
                sideRef.current = side;
                onDragOverChip(operator.id, side);
            }}
            onDragLeave={() => {
                if (!onDragOverChip) return;
                centerXRef.current = null;
                sideRef.current = null;
            }}
        >
            <OperatorAvatar charId={operator.id} name={operator.name} />
            <span className={styles.opRarity} aria-hidden="true" />
        </button>
    );
}
