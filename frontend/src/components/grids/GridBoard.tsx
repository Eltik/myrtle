import { XIcon } from "lucide-react";
import { type CSSProperties, type KeyboardEvent, useRef, useState } from "react";
import { useEdgeFade } from "#/components/tier-lists/edit/FacetFilter";
import { EntityAvatar } from "#/components/tier-lists/entities";
import { useEntityLabels } from "#/components/tier-lists/kinds";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { truncateCodePoints } from "#/lib/markdown/sanitize-input";
import { compactGridSize } from "./compact";
import type { messages } from "./GridBoard.messages";
import styles from "./GridBoard.module.css";
import { cellPosition, GRID_LABEL_MAX } from "./shared";
import type { IGridEditCell } from "./state";

// The board both the view page and the editor draw. In the editor each cell's
// art opens the picker, its strip edits the label in place, a filled cell's
// corner button (or Delete / Backspace on its art) removes the pick at once,
// and a cell dragged onto another swaps the two. The view draws the same
// cells inert.

const CELL_DRAG_MIME = "application/x-grid-cell";

export interface IGridBoardEditor {
    onPick: (index: number) => void;
    /** Remove the cell's pick, keeping its label. No confirmation: the save is the commit. */
    onClear: (index: number) => void;
    onLabelChange: (index: number, label: string) => void;
    onSwap: (from: number, to: number) => void;
}

interface IGridBoardProps {
    title: string;
    rows: number;
    cols: number;
    /** Row-major, `rows * cols` long. */
    cells: IGridEditCell[];
    editor?: IGridBoardEditor;
    /**
     * `compact` draws the board inside another surface (a profile showcase card):
     * no title (the host shows it), tighter padding, and cells sized to a height
     * budget rather than the page width. Read-only. Omitted, the board is the
     * grid page's, unchanged.
     */
    size?: "full" | "compact";
}

/** How large the strip's text is: fewer columns, bigger cells, bigger words. */
function density(cols: number): "large" | "medium" | "small" {
    if (cols <= 3) return "large";
    if (cols <= 6) return "medium";
    return "small";
}

export function GridBoard({ title, rows, cols, cells, editor, size = "full" }: IGridBoardProps) {
    const t: TypedT<typeof messages> = useT("grids");
    const [dragFrom, setDragFrom] = useState<number | null>(null);
    const [dropOver, setDropOver] = useState<number | null>(null);
    // Past six columns a phone scrolls the board inside itself; the faded edge says there is more.
    const fade = useEdgeFade<HTMLDivElement>();
    const shownTitle = title || t("board.untitled");
    const compact = size === "compact" ? compactGridSize(rows, cols) : null;
    const boardStyle = compact ? ({ "--cell-max": `${compact.cellPx}px`, "--cell-gap": `${compact.gapPx}px`, "--label-size": `${compact.labelPx}px`, maxWidth: `${compact.boardMaxPx}px` } as CSSProperties) : undefined;

    return (
        <section className={styles.board} data-density={density(cols)} data-size={compact ? "compact" : undefined} style={boardStyle} aria-label={t("board.label", { title: shownTitle, rows, cols })}>
            {!compact && <h2 className={styles.title}>{shownTitle}</h2>}
            <div ref={fade.ref} style={fade.style} className={styles.scroller}>
                <div className={styles.grid} style={{ "--cols": cols } as CSSProperties}>
                    {cells.map((cell, index) => (
                        <GridCellView
                            // biome-ignore lint/suspicious/noArrayIndexKey: a cell IS its position; swapping moves contents, not cells
                            key={index}
                            cell={cell}
                            index={index}
                            cols={cols}
                            editor={editor}
                            dragging={dragFrom === index}
                            dropTarget={dropOver === index && dragFrom !== null && dragFrom !== index}
                            onDragFrom={setDragFrom}
                            onDropOver={setDropOver}
                        />
                    ))}
                </div>
            </div>
        </section>
    );
}

interface IGridCellViewProps {
    cell: IGridEditCell;
    index: number;
    cols: number;
    editor?: IGridBoardEditor;
    dragging: boolean;
    dropTarget: boolean;
    onDragFrom: (index: number | null) => void;
    onDropOver: (index: number | null) => void;
}

function GridCellView({ cell, index, cols, editor, dragging, dropTarget, onDragFrom, onDropOver }: IGridCellViewProps) {
    const t: TypedT<typeof messages> = useT("grids");
    const labels = useEntityLabels();
    const picked = cell.kind !== null && cell.id !== null;
    const position = cellPosition(index, cols);
    const entityName = cell.entity ? labels.tileLabel(cell.entity) : null;
    const artRef = useRef<HTMLButtonElement>(null);
    // Set while the clear button is pressed, so a press that wanders does not drag the whole cell.
    const pressingClear = useRef(false);

    const art = cell.entity ? <EntityAvatar entity={cell.entity} face="tile" tone="dark" server={cell.server ?? undefined} /> : editor ? <span className={styles.addItem}>{t("cell.addItem")}</span> : null;

    if (!editor) {
        return (
            <div className={styles.cell}>
                <div className={styles.art} data-empty={!picked || undefined} title={entityName ?? undefined}>
                    {art}
                    {entityName && <span className="sr-only">{entityName}</span>}
                </div>
                <div className={styles.strip}>
                    <span className={styles.stripText} title={cell.label || undefined}>
                        {cell.label}
                    </span>
                </div>
            </div>
        );
    }

    const endDrag = () => {
        onDragFrom(null);
        onDropOver(null);
    };
    const clearLabel = t("cell.clear", { name: entityName ?? cell.id ?? "", ...position });

    return (
        // biome-ignore lint/a11y/noStaticElementInteractions: drag-and-drop swap for a pointer; the cell's two buttons are the keyboard and touch path
        <div
            className={styles.cell}
            data-dragging={dragging || undefined}
            data-drop-target={dropTarget || undefined}
            draggable
            onDragStart={(e) => {
                if (pressingClear.current) {
                    e.preventDefault();
                    return;
                }
                e.dataTransfer.effectAllowed = "move";
                e.dataTransfer.setData(CELL_DRAG_MIME, String(index));
                e.dataTransfer.setData("text/plain", cell.label);
                onDragFrom(index);
            }}
            onDragEnd={endDrag}
            onDragOver={(e) => {
                if (!e.dataTransfer.types.includes(CELL_DRAG_MIME)) return;
                e.preventDefault();
                e.dataTransfer.dropEffect = "move";
                onDropOver(index);
            }}
            onDragLeave={() => onDropOver(null)}
            onDrop={(e) => {
                const from = Number(e.dataTransfer.getData(CELL_DRAG_MIME));
                endDrag();
                if (!e.dataTransfer.types.includes(CELL_DRAG_MIME) || !Number.isInteger(from)) return;
                e.preventDefault();
                editor.onSwap(from, index);
            }}
        >
            <button
                ref={artRef}
                type="button"
                className={styles.art}
                data-empty={!picked || undefined}
                onClick={() => editor.onPick(index)}
                onKeyDown={(e) => {
                    if (!picked || (e.key !== "Delete" && e.key !== "Backspace")) return;
                    e.preventDefault();
                    editor.onClear(index);
                }}
                aria-label={entityName ? t("cell.change", { name: entityName, ...position }) : t("cell.pick", position)}
                aria-keyshortcuts={picked ? "Delete Backspace" : undefined}
                title={entityName ?? undefined}
            >
                {art}
            </button>
            {picked && (
                <button
                    type="button"
                    className={styles.clear}
                    draggable={false}
                    onPointerDown={(e) => {
                        e.stopPropagation();
                        pressingClear.current = true;
                    }}
                    onPointerUp={() => {
                        pressingClear.current = false;
                    }}
                    onPointerCancel={() => {
                        pressingClear.current = false;
                    }}
                    onDragStart={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                    }}
                    onClick={(e) => {
                        e.stopPropagation();
                        pressingClear.current = false;
                        editor.onClear(index);
                        // The button goes with the pick; keep focus on the cell.
                        artRef.current?.focus();
                    }}
                    aria-label={clearLabel}
                    title={clearLabel}
                >
                    <XIcon aria-hidden="true" />
                </button>
            )}
            <LabelStrip label={cell.label} position={position} onChange={(label) => editor.onLabelChange(index, label)} />
        </div>
    );
}

interface ILabelStripProps {
    label: string;
    position: { row: number; col: number };
    onChange: (label: string) => void;
}

/** The strip as a button; clicked, it becomes an input. Enter or leaving commits, Escape puts the old label back. */
function LabelStrip({ label, position, onChange }: ILabelStripProps) {
    const t: TypedT<typeof messages> = useT("grids");
    const [draft, setDraft] = useState<string | null>(null);

    if (draft === null) {
        return (
            <button type="button" className={styles.strip} onClick={() => setDraft(label)} aria-label={t("cell.editLabel", { label: label || t("cell.noLabel"), ...position })}>
                {label ? (
                    <span className={styles.stripText} title={label}>
                        {label}
                    </span>
                ) : (
                    <span className={`${styles.stripText} ${styles.stripPlaceholder}`}>{t("cell.labelPlaceholder")}</span>
                )}
            </button>
        );
    }

    const commit = () => {
        if (draft !== label) onChange(draft);
        setDraft(null);
    };
    const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
        if (e.key === "Enter") {
            e.preventDefault();
            commit();
        } else if (e.key === "Escape") {
            e.preventDefault();
            setDraft(null);
        }
    };

    return (
        <div className={styles.strip}>
            <input
                className={styles.stripInput}
                value={draft}
                onChange={(e) => setDraft(truncateCodePoints(e.target.value, GRID_LABEL_MAX))}
                onBlur={commit}
                onKeyDown={onKeyDown}
                aria-label={t("cell.labelInput", position)}
                placeholder={t("cell.labelPlaceholder")}
                // biome-ignore lint/a11y/noAutofocus: the input replaces the strip the user just clicked
                autoFocus
            />
        </div>
    );
}
