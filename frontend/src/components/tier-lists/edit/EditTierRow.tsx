import { ChevronDownIcon, ChevronUpIcon, SettingsIcon, XIcon } from "lucide-react";
import { Fragment, useCallback, useRef, useState } from "react";
import { Button } from "#/components/ui/button";
import type { ITierOperator } from "#/lib/api/tier-lists";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { readableTextColor } from "../detail/contrast";
import { hasOperatorDrag, readOperatorDrag } from "./dnd";
import { useTierDropIndex } from "./drag-controller";
import { EditableOpTile } from "./EditableOpTile";
import styles from "./Editor.module.css";
import type { messages } from "./EditTierRow.messages";
import type { IEditTier } from "./state";

interface IEditTierRowProps {
    tier: IEditTier;
    operators: (ITierOperator | undefined)[];
    notedOperatorIds: Set<string>;
    canMoveUp: boolean;
    canMoveDown: boolean;
    onMoveUp: () => void;
    onMoveDown: () => void;
    onOpenSettings: () => void;
    onPlace: (operatorId: string, tierId: string, index: number) => void;
    onUnplace: (operatorId: string) => void;
    onActivateOperator: (operator: ITierOperator) => void;
}

export function EditTierRow({ tier, operators, notedOperatorIds, canMoveUp, canMoveDown, onMoveUp, onMoveDown, onOpenSettings, onPlace, onUnplace, onActivateOperator }: IEditTierRowProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const textColor = readableTextColor(tier.color);
    const touchDropIndex = useTierDropIndex(tier.id);
    const [mouseDropIndex, setMouseDropIndex] = useState<number | null>(null);
    const dropAreaRef = useRef<HTMLUListElement | null>(null);

    const lastIndex = tier.operatorIds.length;
    const dropIndex = touchDropIndex ?? mouseDropIndex;

    const pendingIdxRef = useRef<number | null>(null);
    const rafRef = useRef<number | null>(null);

    const flushIdx = useCallback(() => {
        rafRef.current = null;
        const next = pendingIdxRef.current;
        pendingIdxRef.current = null;
        if (next !== null) setMouseDropIndex(next);
    }, []);

    const handleOver = useCallback(
        (e: React.DragEvent) => {
            if (!hasOperatorDrag(e)) return;
            e.preventDefault();
            e.dataTransfer.dropEffect = "move";
            if (mouseDropIndex === null) setMouseDropIndex(lastIndex);
        },
        [mouseDropIndex, lastIndex],
    );

    const handleLeave = useCallback((e: React.DragEvent) => {
        const related = e.relatedTarget as Node | null;
        if (related && dropAreaRef.current?.contains(related)) return;
        setMouseDropIndex(null);
    }, []);

    const handleDrop = useCallback(
        (e: React.DragEvent) => {
            const payload = readOperatorDrag(e);
            setMouseDropIndex(null);
            if (!payload) return;
            e.preventDefault();
            const index = mouseDropIndex ?? lastIndex;
            onPlace(payload.operatorId, tier.id, index);
        },
        [mouseDropIndex, lastIndex, onPlace, tier.id],
    );

    const handleChipDragOver = useCallback(
        (operatorId: string, side: "before" | "after") => {
            const idx = tier.operatorIds.indexOf(operatorId);
            const next = idx < 0 ? lastIndex : side === "before" ? idx : idx + 1;
            pendingIdxRef.current = next;
            if (rafRef.current === null) rafRef.current = requestAnimationFrame(flushIdx);
        },
        [lastIndex, tier.operatorIds, flushIdx],
    );

    const isEmpty = operators.length === 0;
    const labelledById = `tier-edit-${tier.id}-label`;
    const showMarker = (i: number) => dropIndex === i;

    return (
        <section
            className={styles.row}
            style={{
                ["--row-color" as string]: tier.color,
                ["--tier-fg" as string]: textColor,
                ["--tier-shadow" as string]: textColor === "white" ? "0 1px 0 oklch(0 0 0 / 0.3)" : "0 1px 0 oklch(1 0 0 / 0.5)",
            }}
            aria-labelledby={labelledById}
        >
            <button id={labelledById} type="button" className={styles.label} onClick={onOpenSettings} aria-label={t("edit.row.editTier", { name: tier.name })}>
                <span>{tier.name}</span>
            </button>

            <ul ref={dropAreaRef} data-tl-drop-tier={tier.id} className={styles.dropArea} data-empty={isEmpty || undefined} data-over={dropIndex !== null || undefined} onDragOver={handleOver} onDragLeave={handleLeave} onDrop={handleDrop} aria-label={t("edit.row.dropArea", { name: tier.name })}>
                {operators.map((op, i) => {
                    if (!op) return null;
                    return (
                        <Fragment key={op.id}>
                            <li className={styles.dropMarker} data-active={showMarker(i) || undefined} aria-hidden="true" />
                            <PlacedChip operator={op} tierName={tier.name} tierLabelId={labelledById} hasNote={notedOperatorIds.has(op.id)} onRemove={onUnplace} onDragOverChip={handleChipDragOver} onActivate={onActivateOperator} />
                        </Fragment>
                    );
                })}
                <li className={styles.dropMarker} data-active={showMarker(lastIndex) || undefined} aria-hidden="true" />
            </ul>

            <div className={styles.rowActions} role="toolbar" aria-label={t("edit.row.actions", { name: tier.name })}>
                <Button type="button" size="icon-xs" variant="outline" onClick={onMoveUp} disabled={!canMoveUp} aria-label={t("edit.row.moveUp")}>
                    <ChevronUpIcon />
                </Button>
                <Button type="button" size="icon-xs" variant="outline" onClick={onOpenSettings} aria-label={t("edit.row.settings")}>
                    <SettingsIcon />
                </Button>
                <Button type="button" size="icon-xs" variant="outline" onClick={onMoveDown} disabled={!canMoveDown} aria-label={t("edit.row.moveDown")}>
                    <ChevronDownIcon />
                </Button>
            </div>
        </section>
    );
}

interface IPlacedChipProps {
    operator: ITierOperator;
    tierName: string;
    /** Focus target when the chip being removed was the last one in its row. */
    tierLabelId: string;
    hasNote: boolean;
    onRemove: (operatorId: string) => void;
    onDragOverChip: (operatorId: string, side: "before" | "after") => void;
    onActivate: (operator: ITierOperator) => void;
}

/**
 * One placed tile plus its remove cross. The cross is a sibling of the tile's
 * button, not a child, because nested controls are invalid HTML; Delete and
 * Backspace on the focused tile remove it too.
 */
function PlacedChip({ operator, tierName, tierLabelId, hasNote, onRemove, onDragOverChip, onActivate }: IPlacedChipProps) {
    const t: TypedT<typeof messages> = useT("tierLists");

    const handleKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>) => {
        if (e.key !== "Delete" && e.key !== "Backspace") return;
        e.preventDefault();
        focusNeighbourChip(e.currentTarget, tierLabelId);
        onRemove(operator.id);
    };

    return (
        <li className={styles.chip}>
            <EditableOpTile operator={operator} hasNote={hasNote} onDragOverChip={onDragOverChip} onActivate={onActivate} onKeyDown={handleKeyDown} />
            <button type="button" className={styles.chipRemove} onClick={() => onRemove(operator.id)} aria-label={t("edit.row.removeOperator", { name: operator.name, tier: tierName })} title={t("edit.row.removeOperatorTitle")}>
                <XIcon aria-hidden="true" />
            </button>
        </li>
    );
}

/**
 * Removing the focused tile unmounts it, which would drop focus to `body`.
 * Move focus first: to the next tile in the row, else the previous one, else
 * the tier label when the row is about to be empty. Tiles are keyed by
 * operator id, so the neighbour survives the re-render.
 */
function focusNeighbourChip(tile: HTMLElement, tierLabelId: string) {
    const row = tile.closest("li")?.parentElement;
    if (!row) return;
    const chips = Array.from(row.querySelectorAll<HTMLElement>("[data-tl-chip-id]"));
    const idx = chips.indexOf(tile);
    const next = chips[idx + 1] ?? chips[idx - 1] ?? document.getElementById(tierLabelId);
    next?.focus();
}
