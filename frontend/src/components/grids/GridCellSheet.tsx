import { ChevronLeftIcon, ChevronRightIcon, PencilIcon, PlusIcon, Trash2Icon } from "lucide-react";
import { type ReactNode, type RefObject, useId, useRef } from "react";
import { useEntityLabels } from "#/components/tier-lists/kinds";
import { Button } from "#/components/ui/button";
import { Dialog, DialogBackdrop, DialogDescription, DialogPortal, DialogPrimitive, DialogTitle } from "#/components/ui/dialog";
import { Input } from "#/components/ui/input";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { truncateCodePoints } from "#/lib/markdown/sanitize-input";
import { CellHeroArt } from "./CellHeroArt";
import type { messages } from "./GridCellSheet.messages";
import { cellPosition, GRID_LABEL_MAX } from "./shared";
import type { IGridEditCell } from "./state";

// The grid editor's cell, full screen, on a phone: a 52 px cell is too small
// to read, to type a label into, or to aim at its corner button. Every change
// goes through the editor's own actions at once, the same reducer the board
// uses, so there is no draft to commit: Done only closes. Prev and next walk
// every cell row by row, empty ones included, since filling them is the job.

type SheetT = TypedT<typeof messages>;

interface IGridCellSheetProps {
    cells: readonly IGridEditCell[];
    cols: number;
    /** The cell open, `null` when closed. */
    index: number | null;
    onIndexChange: (index: number) => void;
    onClose: () => void;
    onPick: (index: number) => void;
    onClear: (index: number) => void;
    onLabelChange: (index: number, label: string) => void;
    /** The picker, rendered inside so it opens as a dialog nested over this one. */
    children?: ReactNode;
}

export function GridCellSheet({ cells, cols, index, onIndexChange, onClose, onPick, onClear, onLabelChange, children }: IGridCellSheetProps) {
    const t: SheetT = useT("grids");
    const doneRef = useRef<HTMLButtonElement>(null);
    const lastShown = useRef<number | null>(null);
    if (index !== null) lastShown.current = index;
    const shown = index ?? lastShown.current;
    const cell = shown === null ? undefined : cells[shown];

    return (
        <Dialog open={index !== null} onOpenChange={(open) => !open && onClose()}>
            <DialogPortal>
                <DialogBackdrop />
                <DialogPrimitive.Popup
                    // Focus lands on Done, not the label field: a field would raise the keyboard over the preview at once.
                    initialFocus={doneRef}
                    finalFocus={() => (lastShown.current === null ? true : (document.querySelector<HTMLElement>(`[data-cell-index="${lastShown.current}"]`) ?? true))}
                    className="fixed inset-0 z-50 flex flex-col bg-background text-foreground outline-none transition-[opacity,translate] duration-200 ease-out data-ending-style:translate-y-4 data-starting-style:translate-y-4 data-ending-style:opacity-0 data-starting-style:opacity-0 motion-reduce:transition-none"
                >
                    {cell && shown !== null && <SheetContent t={t} cell={cell} index={shown} cols={cols} total={cells.length} doneRef={doneRef} onIndexChange={onIndexChange} onPick={onPick} onClear={onClear} onLabelChange={onLabelChange} />}
                    {children}
                </DialogPrimitive.Popup>
            </DialogPortal>
        </Dialog>
    );
}

interface ISheetContentProps {
    t: SheetT;
    cell: IGridEditCell;
    index: number;
    cols: number;
    total: number;
    doneRef: RefObject<HTMLButtonElement | null>;
    onIndexChange: (index: number) => void;
    onPick: (index: number) => void;
    onClear: (index: number) => void;
    onLabelChange: (index: number, label: string) => void;
}

function SheetContent({ t, cell, index, cols, total, doneRef, onIndexChange, onPick, onClear, onLabelChange }: ISheetContentProps) {
    const labels = useEntityLabels();
    const labelId = useId();
    const counterId = useId();
    const position = cellPosition(index, cols);
    const entity = cell.entity;
    const picked = cell.kind !== null;
    const used = Array.from(cell.label).length;

    return (
        <>
            <div className="flex items-center gap-3 border-border border-b px-4 pt-[max(0.75rem,env(safe-area-inset-top,0px))] pb-3">
                <div className="flex min-w-0 flex-1 flex-col gap-0.5">
                    <DialogTitle className="text-lg">{t("sheet.title", position)}</DialogTitle>
                    <span className="font-mono text-[10.5px] text-muted-foreground uppercase tabular-nums tracking-[0.12em]">{t("sheet.count", { n: index + 1, total })}</span>
                </div>
                <DialogDescription className="sr-only">{t("sheet.description")}</DialogDescription>
                <DialogPrimitive.Close ref={doneRef} render={<Button type="button" className="min-w-20" />}>
                    {t("sheet.done")}
                </DialogPrimitive.Close>
            </div>

            <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-4 py-4">
                <div className="relative flex h-[34dvh] min-h-44 items-center justify-center overflow-hidden rounded-xl bg-[oklch(0.13_0.004_285)] text-white">
                    {entity ? (
                        <CellHeroArt entity={entity} server={cell.server} size="sheet" />
                    ) : (
                        <button type="button" onClick={() => onPick(index)} className="flex size-full cursor-pointer flex-col items-center justify-center gap-2 font-sans text-sm text-white/60">
                            <PlusIcon className="size-7" aria-hidden="true" />
                            {t("sheet.empty")}
                        </button>
                    )}
                </div>

                <div className="flex items-center gap-2">
                    <div className="flex min-w-0 flex-1 flex-col">
                        {entity && (
                            <>
                                {entity.resolved && <span className="font-bold font-mono text-[10px] text-muted-foreground uppercase tracking-[0.14em]">{labels.singular(entity.kind)}</span>}
                                <span className="truncate font-sans font-semibold text-base">{entity.resolved ? entity.name : entity.id}</span>
                            </>
                        )}
                    </div>
                    <Button type="button" variant="outline" onClick={() => onPick(index)}>
                        {picked ? <PencilIcon /> : <PlusIcon />}
                        {picked ? t("sheet.change") : t("sheet.pick")}
                    </Button>
                    <Button type="button" variant="ghost" onClick={() => onClear(index)} disabled={!picked}>
                        <Trash2Icon />
                        {t("sheet.clear")}
                    </Button>
                </div>

                <div className="flex flex-col gap-1.5">
                    <label htmlFor={labelId} className="font-medium font-sans text-sm">
                        {t("sheet.label")}
                    </label>
                    {/* 16 px text, so iOS does not zoom the page into the field. */}
                    <Input
                        id={labelId}
                        value={cell.label}
                        onChange={(e) => onLabelChange(index, truncateCodePoints((e.target as HTMLInputElement).value, GRID_LABEL_MAX))}
                        onKeyDown={(e) => {
                            if (e.key === "Enter") (e.target as HTMLInputElement).blur();
                        }}
                        enterKeyHint="done"
                        aria-describedby={counterId}
                        size="lg"
                        className="[&_input]:text-base"
                    />
                    <span id={counterId} className="self-end font-mono text-[11px] text-muted-foreground tabular-nums">
                        {t("sheet.counter", { count: used, max: GRID_LABEL_MAX })}
                    </span>
                </div>
            </div>

            <div className="flex items-center justify-between gap-3 border-border border-t px-4 pt-3 pb-[max(0.75rem,env(safe-area-inset-bottom,0px))]">
                <Button type="button" variant="outline" size="lg" onClick={() => onIndexChange(index - 1)} disabled={index === 0}>
                    <ChevronLeftIcon />
                    {t("sheet.prev")}
                </Button>
                <Button type="button" variant="outline" size="lg" onClick={() => onIndexChange(index + 1)} disabled={index >= total - 1}>
                    {t("sheet.next")}
                    <ChevronRightIcon />
                </Button>
            </div>
        </>
    );
}
