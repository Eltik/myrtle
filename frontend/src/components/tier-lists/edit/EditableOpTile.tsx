import type { ComponentPropsWithoutRef, Ref } from "react";
import { useRef, useState } from "react";
import { entityOwner, type ITierEntity, isOperatorEntity } from "#/lib/api/tier-entities";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { EntityAvatar, entityAccent, kindTileAttributes } from "../entities";
import { useEntityLabels } from "../kinds";
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
    const noted = Boolean(hasNote);
    let label: string;
    let tooltip: string;
    if (isOperatorEntity(entity)) {
        label = t("edit.tile.label", { name: entity.name, rarity: entity.rarity, placed: Boolean(placed), noted });
        tooltip = t("edit.tile.title", { name: entity.name, rarity: entity.rarity, noted });
    } else if (!entity.resolved) {
        label = entity.id;
        tooltip = entity.id;
    } else {
        // A skin, module or skill names its operator too: "Stick and Sack" alone does not say whose it is.
        const owner = entityOwner(entity);
        const kind = labels.singular(entity.kind);
        label = owner ? t("edit.tile.entityLabelOwned", { name: entity.name, owner, kind, placed: Boolean(placed), noted }) : t("edit.tile.entityLabel", { name: entity.name, kind, placed: Boolean(placed), noted });
        tooltip = owner ? t("edit.tile.entityTitleOwned", { name: entity.name, owner, noted }) : t("edit.tile.entityTitle", { name: entity.name, noted });
    }
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
            {...kindTileAttributes(entity)}
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
