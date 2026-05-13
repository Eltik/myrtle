import { ChevronDownIcon, ChevronUpIcon, SettingsIcon } from "lucide-react";
import { Fragment, useCallback, useRef, useState } from "react";
import { Button } from "#/components/ui/button";
import type { ITierOperator } from "#/lib/api/tier-lists";
import { readableTextColor } from "../detail/contrast";
import { hasOperatorDrag, readOperatorDrag } from "./dnd";
import { EditableOpTile } from "./EditableOpTile";
import styles from "./Editor.module.css";
import type { IEditTier } from "./state";

interface IEditTierRowProps {
    tier: IEditTier;
    operators: (ITierOperator | undefined)[];
    canMoveUp: boolean;
    canMoveDown: boolean;
    onMoveUp: () => void;
    onMoveDown: () => void;
    onOpenSettings: () => void;
    onPlace: (operatorId: string, tierId: string, index: number) => void;
    onUnplace: (operatorId: string) => void;
    onActivateOperator: (operator: ITierOperator) => void;
}

export function EditTierRow({ tier, operators, canMoveUp, canMoveDown, onMoveUp, onMoveDown, onOpenSettings, onPlace, onActivateOperator }: IEditTierRowProps) {
    const textColor = readableTextColor(tier.color);
    const [dropTargetIndex, setDropTargetIndex] = useState<number | null>(null);
    const dropAreaRef = useRef<HTMLUListElement | null>(null);

    const lastIndex = tier.operatorIds.length;

    const handleOver = useCallback(
        (e: React.DragEvent) => {
            if (!hasOperatorDrag(e)) return;
            e.preventDefault();
            e.dataTransfer.dropEffect = "move";
            if (dropTargetIndex === null) setDropTargetIndex(lastIndex);
        },
        [dropTargetIndex, lastIndex],
    );

    const handleLeave = useCallback((e: React.DragEvent) => {
        const related = e.relatedTarget as Node | null;
        if (related && dropAreaRef.current?.contains(related)) return;
        setDropTargetIndex(null);
    }, []);

    const handleDrop = useCallback(
        (e: React.DragEvent) => {
            const payload = readOperatorDrag(e);
            setDropTargetIndex(null);
            if (!payload) return;
            e.preventDefault();
            const index = dropTargetIndex ?? lastIndex;
            onPlace(payload.operatorId, tier.id, index);
        },
        [dropTargetIndex, lastIndex, onPlace, tier.id],
    );

    const handleChipDragOver = useCallback(
        (operatorId: string, side: "before" | "after") => {
            const idx = tier.operatorIds.indexOf(operatorId);
            if (idx < 0) {
                setDropTargetIndex(lastIndex);
                return;
            }
            setDropTargetIndex(side === "before" ? idx : idx + 1);
        },
        [lastIndex, tier.operatorIds],
    );

    const isEmpty = operators.length === 0;
    const labelledById = `tier-edit-${tier.id}-label`;
    const showMarker = (i: number) => dropTargetIndex === i;

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
            <button id={labelledById} type="button" className={styles.label} onClick={onOpenSettings} aria-label={`Edit tier "${tier.name}"`}>
                <span>{tier.name}</span>
            </button>

            <ul ref={dropAreaRef} className={styles.dropArea} data-empty={isEmpty || undefined} data-over={dropTargetIndex !== null || undefined} onDragOver={handleOver} onDragLeave={handleLeave} onDrop={handleDrop} aria-label={`Operators in tier ${tier.name}`}>
                {operators.map((op, i) => {
                    if (!op) return null;
                    return (
                        <Fragment key={op.id}>
                            <li className={styles.dropMarker} data-active={showMarker(i) || undefined} aria-hidden="true" />
                            <li className="contents">
                                <EditableOpTile operator={op} onDragOverChip={handleChipDragOver} onActivate={onActivateOperator} />
                            </li>
                        </Fragment>
                    );
                })}
                <li className={styles.dropMarker} data-active={showMarker(lastIndex) || undefined} aria-hidden="true" />
            </ul>

            <div className={styles.rowActions} role="toolbar" aria-label={`Tier ${tier.name} actions`}>
                <Button type="button" size="icon-xs" variant="outline" onClick={onMoveUp} disabled={!canMoveUp} aria-label="Move tier up">
                    <ChevronUpIcon />
                </Button>
                <Button type="button" size="icon-xs" variant="outline" onClick={onOpenSettings} aria-label="Tier settings">
                    <SettingsIcon />
                </Button>
                <Button type="button" size="icon-xs" variant="outline" onClick={onMoveDown} disabled={!canMoveDown} aria-label="Move tier down">
                    <ChevronDownIcon />
                </Button>
            </div>
        </section>
    );
}
