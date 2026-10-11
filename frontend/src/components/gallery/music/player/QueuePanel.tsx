import { Disc3Icon, GripVerticalIcon, ListPlusIcon, Repeat1Icon, RepeatIcon, Settings2Icon, ShuffleIcon, SkipBackIcon, Trash2Icon, XIcon } from "lucide-react";
import type React from "react";
import { memo, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { Sheet, SheetDescription, SheetHeader, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { toastManager } from "#/components/ui/toast";
import { useT } from "#/lib/i18n";
import { cn } from "#/lib/utils";
import { sizedCover } from "../impl/cover";
import { createPlaylist } from "../impl/library/store";
import type { ITrack } from "../impl/model";
import { clearQueue, cycleRepeat, openPanel, playAt, playerStore, previous, removeFromQueue, reorderQueue, setSettingsOpen, toggleShuffle, usePlayer } from "./engine";
import { IconButton, type PlayerT, subLine, useKicker } from "./parts";
import { moveItem } from "./queue";

/** Near the list's top or bottom edge a drag scrolls the list, this many pixels from the edge, this far per move. */
const EDGE_ZONE_PX = 32;
const EDGE_STEP_PX = 8;

export function QueuePanel(): React.ReactElement {
    const t: PlayerT = useT("gallery");
    const open = usePlayer((s) => s.panel === "queue");
    const source = usePlayer((s) => s.source);
    const kicker = useKicker(source);
    const count = usePlayer((s) => s.queue.length);
    const [saving, setSaving] = useState(false);
    const [clearing, setClearing] = useState(false);

    return (
        <Sheet onOpenChange={(o) => openPanel(o ? "queue" : null)} open={open}>
            <SheetPopup className="max-w-95" side="right">
                <SheetHeader className="gap-1 pr-14 pb-3">
                    <SheetTitle>{t("music.queue.title")}</SheetTitle>
                    <SheetDescription className="truncate">{kicker}</SheetDescription>
                    <PhoneTransport t={t} />
                    {count > 0 && (
                        <div className="mt-2 flex flex-wrap gap-2">
                            <Button onClick={() => setSaving(true)} size="sm" variant="outline">
                                <ListPlusIcon aria-hidden="true" />
                                {t("music.queue.save")}
                            </Button>
                            <Button onClick={() => setClearing(true)} size="sm" variant="ghost">
                                <Trash2Icon aria-hidden="true" />
                                {t("music.queue.clear")}
                            </Button>
                        </div>
                    )}
                </SheetHeader>
                <QueueList t={t} />
                <SaveDialog onOpenChange={setSaving} open={saving} t={t} />
                <ClearDialog onOpenChange={setClearing} open={clearing} t={t} />
            </SheetPopup>
        </Sheet>
    );
}

/**
 * The controls the bar drops under 640 px (shuffle, previous, repeat,
 * settings), so a phone still reaches them: the bar's track area opens this
 * sheet. Hidden from 640 px up, where the bar shows them itself.
 */
function PhoneTransport({ t }: { t: PlayerT }): React.ReactElement {
    const shuffle = usePlayer((s) => s.shuffle);
    const repeat = usePlayer((s) => s.repeat);
    const repeatLabel = repeat === "one" ? t("music.player.repeat.one") : repeat === "all" ? t("music.player.repeat.all") : t("music.player.repeat.off");
    return (
        <div className="-ml-2.5 flex items-center gap-1 sm:hidden">
            <IconButton label={shuffle ? t("music.player.shuffle.on") : t("music.player.shuffle.off")} onClick={toggleShuffle} pressed={shuffle} tooltip={false}>
                <ShuffleIcon aria-hidden="true" />
            </IconButton>
            <IconButton label={t("music.player.previous")} onClick={previous} tooltip={false}>
                <SkipBackIcon aria-hidden="true" />
            </IconButton>
            <IconButton label={repeatLabel} onClick={cycleRepeat} pressed={repeat !== "off"} tooltip={false}>
                {repeat === "one" ? <Repeat1Icon aria-hidden="true" /> : <RepeatIcon aria-hidden="true" />}
            </IconButton>
            <IconButton label={t("music.player.settings")} onClick={() => setSettingsOpen(true)} tooltip={false}>
                <Settings2Icon aria-hidden="true" />
            </IconButton>
        </div>
    );
}

interface IDrag {
    from: number;
    over: number;
    pointerId: number;
    rowHeight: number;
}

function QueueList({ t }: { t: PlayerT }): React.ReactElement {
    const queue = usePlayer((s) => s.queue);
    const index = usePlayer((s) => s.index);
    const houseArtist = usePlayer((s) => s.houseArtist);
    const listRef = useRef<HTMLOListElement>(null);
    const scrollRef = useRef<HTMLDivElement>(null);
    const [drag, setDrag] = useState<IDrag | null>(null);
    const focusAfterMove = useRef<number | null>(null);
    // Rows are memoised, so they get handlers that never change identity and call the latest closures.
    const latest = useRef<IRowHandlers | null>(null);
    const handlers = useMemo<IRowHandlers>(
        () => ({
            down: (e, from) => latest.current?.down(e, from),
            move: (e) => latest.current?.move(e),
            up: (e) => latest.current?.up(e),
            key: (e, i) => latest.current?.key(e, i),
        }),
        [],
    );

    // The current row scrolls to the centre when it changes and when the panel opens.
    useEffect(() => {
        if (index < 0) return;
        const row = listRef.current?.querySelector<HTMLElement>(`[data-row="${index}"]`);
        row?.scrollIntoView?.({ block: "center" });
    }, [index]);

    // A keyboard move keeps focus on the row that moved.
    useLayoutEffect(() => {
        const at = focusAfterMove.current;
        if (at === null) return;
        focusAfterMove.current = null;
        listRef.current?.querySelector<HTMLElement>(`[data-row="${at}"] [data-row-main]`)?.focus();
    });

    if (queue.length === 0) return <p className="px-6 pb-6 text-muted-foreground text-sm">{t("music.queue.empty")}</p>;

    // While dragging, the rows draw as if the move had happened, so the gap follows the pointer.
    const shown = drag ? moveItem({ items: queue.map((tr, i) => ({ tr, i })), index: -1, order: [] }, drag.from, drag.over, false).items : queue.map((tr, i) => ({ tr, i }));

    const startDrag = (e: React.PointerEvent<HTMLElement>, from: number) => {
        if (e.button !== 0) return;
        const row = listRef.current?.querySelector<HTMLElement>(`[data-row="${from}"]`);
        const rowHeight = row?.offsetHeight || 56;
        e.currentTarget.setPointerCapture?.(e.pointerId);
        e.preventDefault();
        setDrag({ from, over: from, pointerId: e.pointerId, rowHeight });
    };

    const moveDrag = (e: React.PointerEvent<HTMLElement>) => {
        if (!drag || e.pointerId !== drag.pointerId) return;
        const list = listRef.current;
        const scroller = scrollRef.current;
        if (!list) return;
        const rect = list.getBoundingClientRect();
        const over = Math.max(0, Math.min(queue.length - 1, Math.floor((e.clientY - rect.top) / drag.rowHeight)));
        if (over !== drag.over) setDrag({ ...drag, over });
        if (scroller) {
            const box = scroller.getBoundingClientRect();
            if (e.clientY < box.top + EDGE_ZONE_PX) scroller.scrollTop -= EDGE_STEP_PX;
            else if (e.clientY > box.bottom - EDGE_ZONE_PX) scroller.scrollTop += EDGE_STEP_PX;
        }
    };

    const endDrag = (e: React.PointerEvent<HTMLElement>) => {
        if (!drag || e.pointerId !== drag.pointerId) return;
        const { from, over } = drag;
        setDrag(null);
        if (from !== over) reorderQueue(from, over);
    };

    const onRowKey = (e: React.KeyboardEvent<HTMLElement>, i: number) => {
        if (!e.altKey || (e.key !== "ArrowUp" && e.key !== "ArrowDown")) return;
        const to = e.key === "ArrowUp" ? i - 1 : i + 1;
        if (to < 0 || to >= queue.length) return;
        e.preventDefault();
        focusAfterMove.current = to;
        reorderQueue(i, to);
    };

    latest.current = { down: startDrag, move: moveDrag, up: endDrag, key: onRowKey };

    return (
        <div className="min-h-0 flex-1 overflow-y-auto px-3 pb-4" ref={scrollRef}>
            <ol className="flex flex-col" ref={listRef}>
                {shown.map(({ tr, i }, at) => (
                    <QueueRow current={i === index} houseArtist={houseArtist} index={i} key={`${tr.id}-${i}`} lifted={drag?.from === i} onGripDown={handlers.down} onGripMove={handlers.move} onGripUp={handlers.up} onKey={handlers.key} position={at} t={t} total={queue.length} track={tr} />
                ))}
            </ol>
        </div>
    );
}

interface IRowHandlers {
    down: (e: React.PointerEvent<HTMLElement>, from: number) => void;
    move: (e: React.PointerEvent<HTMLElement>) => void;
    up: (e: React.PointerEvent<HTMLElement>) => void;
    key: (e: React.KeyboardEvent<HTMLElement>, i: number) => void;
}

/**
 * One queue row. Memoised: a drag step re-renders the whole list in its new
 * order, and with a full library queued (900+ rows) that was every row on
 * every step; now only the rows whose position or lift changed. Off-screen
 * rows skip layout and paint (`content-visibility`, a 56 px estimate that
 * matches `h-14`); the lifted row opts out, since paint containment would
 * clip its shadow.
 */
const QueueRow = memo(function QueueRow({
    track,
    index,
    position,
    total,
    current,
    lifted,
    houseArtist,
    t,
    onGripDown,
    onGripMove,
    onGripUp,
    onKey,
}: {
    track: ITrack;
    index: number;
    position: number;
    total: number;
    current: boolean;
    lifted: boolean;
    houseArtist: string | null;
    t: PlayerT;
    onGripDown: (e: React.PointerEvent<HTMLElement>, from: number) => void;
    onGripMove: (e: React.PointerEvent<HTMLElement>) => void;
    onGripUp: (e: React.PointerEvent<HTMLElement>) => void;
    onKey: (e: React.KeyboardEvent<HTMLElement>, i: number) => void;
}): React.ReactElement {
    const title = track.title;
    const label = current ? t("music.queue.rowCurrent", { title, position: index + 1, total }) : t("music.queue.row", { title, position: index + 1, total });
    return (
        <li
            className={cn("group relative flex h-14 items-center gap-2 rounded-lg pr-1 transition-colors", current ? "bg-primary/10" : "hover:bg-secondary/50", lifted ? "z-10 bg-card shadow-lg ring-1 ring-border" : "[contain-intrinsic-size:auto_56px] [content-visibility:auto]")}
            data-position={position}
            data-row={index}
        >
            <span
                aria-hidden="true"
                className="flex h-full w-6 shrink-0 cursor-grab touch-none items-center justify-center text-muted-foreground/60 hover:text-foreground active:cursor-grabbing"
                onPointerCancel={onGripUp}
                onPointerDown={(e) => onGripDown(e, index)}
                onPointerMove={onGripMove}
                onPointerUp={onGripUp}
                title={t("music.queue.grip")}
            >
                <GripVerticalIcon className="size-4" />
            </span>
            <button aria-label={label} className="flex min-w-0 flex-1 cursor-pointer items-center gap-2.5 rounded-md py-1 text-left outline-none focus-visible:ring-2 focus-visible:ring-ring/40" data-row-main="" onClick={() => playAt(index)} onKeyDown={(e) => onKey(e, index)} type="button">
                <span className="relative size-10 shrink-0 overflow-hidden rounded border border-border bg-secondary/50">
                    {track.coverUrl ? <img referrerPolicy="no-referrer" alt="" className="size-full object-cover" decoding="async" loading="lazy" src={sizedCover(track.coverUrl, 40)} /> : <Disc3Icon aria-hidden="true" className="absolute inset-0 m-auto size-5 text-muted-foreground" />}
                </span>
                <span className="flex min-w-0 flex-col leading-tight">
                    <span className={cn("truncate text-[13px]", current ? "font-semibold text-primary" : "text-foreground")}>{title}</span>
                    <span className="truncate text-[11.5px] text-muted-foreground">{subLine(track, houseArtist)}</span>
                </span>
            </button>
            <button
                aria-label={t("music.queue.remove", { title })}
                className="inline-flex pointer-coarse:size-11 size-8 shrink-0 cursor-pointer items-center justify-center rounded-full text-muted-foreground opacity-70 outline-none hover:bg-secondary hover:text-foreground focus-visible:opacity-100 focus-visible:ring-2 focus-visible:ring-ring/40 group-hover:opacity-100 max-sm:min-h-11 max-sm:min-w-11"
                onClick={() => removeFromQueue(index)}
                type="button"
            >
                <XIcon aria-hidden="true" className="size-4" />
            </button>
        </li>
    );
});

function SaveDialog({ open, onOpenChange, t }: { open: boolean; onOpenChange: (o: boolean) => void; t: PlayerT }): React.ReactElement {
    const [name, setName] = useState("");
    const [busy, setBusy] = useState(false);
    const trimmed = name.trim();

    const save = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!trimmed || busy) return;
        setBusy(true);
        const ids = playerStore.state.queue.map((tr) => tr.id);
        const created = await createPlaylist(trimmed, ids);
        setBusy(false);
        if (created) {
            toastManager.add({ title: t("music.queue.saved", { name: created.name }), type: "success" });
            setName("");
            onOpenChange(false);
        } else toastManager.add({ title: t("music.queue.saveFailed"), type: "error" });
    };

    return (
        <Dialog onOpenChange={onOpenChange} open={open}>
            <DialogPopup className="max-w-sm">
                <form className="flex min-h-0 flex-col" onSubmit={save}>
                    <DialogHeader>
                        <DialogTitle>{t("music.queue.saveTitle")}</DialogTitle>
                    </DialogHeader>
                    <DialogPanel className="flex flex-col gap-2">
                        <Label htmlFor="music-queue-playlist-name">{t("music.queue.nameLabel")}</Label>
                        <Input autoFocus id="music-queue-playlist-name" maxLength={100} onChange={(e) => setName(e.currentTarget.value)} placeholder={t("music.queue.namePlaceholder")} value={name} />
                    </DialogPanel>
                    <DialogFooter>
                        <DialogClose render={<Button type="button" variant="outline" />}>{t("music.queue.cancel")}</DialogClose>
                        <Button disabled={!trimmed} loading={busy} type="submit">
                            {t("music.queue.saveConfirm")}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogPopup>
        </Dialog>
    );
}

function ClearDialog({ open, onOpenChange, t }: { open: boolean; onOpenChange: (o: boolean) => void; t: PlayerT }): React.ReactElement {
    return (
        <Dialog onOpenChange={onOpenChange} open={open}>
            <DialogPopup className="max-w-sm">
                <DialogHeader>
                    <DialogTitle>{t("music.queue.clearTitle")}</DialogTitle>
                    <DialogDescription>{t("music.queue.clearBody")}</DialogDescription>
                </DialogHeader>
                <DialogFooter>
                    <DialogClose render={<Button variant="outline" />}>{t("music.queue.cancel")}</DialogClose>
                    <Button
                        onClick={() => {
                            onOpenChange(false);
                            clearQueue();
                        }}
                        variant="destructive"
                    >
                        {t("music.queue.clearConfirm")}
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}
