import type { ComponentPropsWithoutRef, Ref } from "react";
import { useRef, useState } from "react";
import { type ITierEntity, isOperatorEntity } from "#/lib/api/tier-entities";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { EntityAvatar, entityAccent, entityShape, useEntityLabels } from "../entities";
import { setEntityDrag } from "./dnd";
import { useIsDragSource, useStartEntityDrag } from "./drag-controller";
import type { messages } from "./EditableOpTile.messages";
import styles from "./Editor.module.css";

type ButtonExtras = Omit<ComponentPropsWithoutRef<"button">, "title" | "onDragStart" | "onDragEnd" | "onClick" | "onPointerDown" | "onPointerMove" | "onDragOver" | "ref" | "className" | "style">;

interface IEditableOpTileProps extends ButtonExtras {
    entity: ITierEntity;
    disabled?: boolean;
    placed?: boolean;
    hasNote?: boolean;
    onActivate?: (entity: ITierEntity) => void;
    onDragStart?: (entityKey: string) => void;
    onDragEnd?: () => void;
    onDragOverChip?: (entityKey: string, side: "before" | "after") => void;
    title?: string;
    ref?: Ref<HTMLButtonElement>;
    className?: string;
}

/** A draggable placement tile in the editor. Operators keep their name-and-stars label; an unresolved placement is labelled by its raw id. */
export function EditableOpTile({ entity, disabled, placed, hasNote, onActivate, onDragStart, onDragEnd, onDragOverChip, title, ref, className, ...rest }: IEditableOpTileProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const labels = useEntityLabels();
    const color = entityAccent(entity);
    const isOperator = isOperatorEntity(entity);
    // An operator keeps exactly the tile it always had; other kinds mark themselves for the pool grid and the wide event shape.
    const kindAttrs = isOperator || !entity.resolved ? {} : { "data-kind": entity.kind, "data-shape": entityShape(entity) };
    const label = isOperatorEntity(entity)
        ? t("edit.tile.label", { name: entity.name, rarity: entity.rarity, placed: Boolean(placed), noted: Boolean(hasNote) })
        : entity.resolved
          ? t("edit.tile.entityLabel", { name: entity.name, kind: labels.kind[entity.kind], placed: Boolean(placed), noted: Boolean(hasNote) })
          : entity.id;
    const tooltip = isOperatorEntity(entity) ? t("edit.tile.title", { name: entity.name, rarity: entity.rarity, noted: Boolean(hasNote) }) : entity.resolved ? t("edit.tile.entityTitle", { name: entity.name, noted: Boolean(hasNote) }) : entity.id;
    const isTouchDragging = useIsDragSource(entity.key);
    const startPress = useStartEntityDrag();
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
            {...kindAttrs}
            ref={ref}
            type="button"
            data-tl-chip-id={entity.key}
            className={cn(styles.opTile, className)}
            style={{ ["--rarity-color" as string]: color }}
            draggable={!disabled}
            data-dragging={isDragging || undefined}
            data-disabled={disabled || undefined}
            aria-disabled={disabled || undefined}
            aria-label={label}
            title={title ?? tooltip}
            onClick={() => {
                if (dragStartedRef.current) {
                    dragStartedRef.current = false;
                    return;
                }
                if (!disabled) onActivate?.(entity);
            }}
            onPointerDown={(e) => {
                if (disabled) return;
                if (e.pointerType === "touch" || e.pointerType === "pen") {
                    dragStartedRef.current = false;
                    startPress(e, entity.key);
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
                setEntityDrag(e, { entityKey: entity.key });
                setMouseDragging(true);
                onDragStart?.(entity.key);
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
                onDragOverChip(entity.key, side);
            }}
            onDragLeave={() => {
                if (!onDragOverChip) return;
                centerXRef.current = null;
                sideRef.current = null;
            }}
        >
            <EntityAvatar entity={entity} face="tile" />
            <span className={styles.opRarity} aria-hidden="true" />
            {hasNote && <span className={styles.opNote} aria-hidden="true" />}
        </button>
    );
}
