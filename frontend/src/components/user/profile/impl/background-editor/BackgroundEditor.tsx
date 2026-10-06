import { ChevronsDownIcon, MonitorIcon, SmartphoneIcon } from "lucide-react";
import { type CSSProperties, memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { Dialog, DialogDescription, DialogPortal, DialogPrimitive, DialogTitle } from "#/components/ui/dialog";
import { Spinner } from "#/components/ui/spinner";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { ProfileBackground } from "#/types/generated/ProfileBackground";
import type { IUserProfile } from "#/types/user";
import { AdjustBar } from "./AdjustBar";
import { ArtBrowser, type IPickedArt } from "./ArtBrowser";
import type { messages } from "./BackgroundEditor.messages";
import { SEGMENT_ITEM } from "./chips";
import { PreviewStage } from "./PreviewStage";
import { defaultPreviewMode, type PreviewMode, previewMaxHeight, previewViewport, STRIP_HEIGHT, STRIP_PREVIEW_MAX, stripShown } from "./preview-geometry";
import { useBackgroundDraft } from "./useBackgroundDraft";

interface IBackgroundEditorProps {
    profile: IUserProfile;
    /** The background saved now, `null` for none. */
    saved: ProfileBackground | null;
    onClose: () => void;
}

/** How long the "zoom in to move" line stays after a drag or key along a dead axis. */
const DEAD_AXIS_MS = 2400;

/** The page's width without its scrollbar, and its height: what the profile lays out at. */
const readViewport = () => ({ width: document.documentElement.clientWidth, height: window.innerHeight });

/**
 * `readViewport`, kept current. Re-read on `resize` and whenever `<html>` changes size,
 * which is also when a page scrollbar comes or goes without the window resizing. The
 * dialog's scroll lock does not narrow or widen it in Chrome: base-ui keeps the gutter
 * (`scrollbar-gutter: stable`), so `clientWidth` read 1485 before and after the lock with
 * a 15 px classic scrollbar (2026-10-06); the observer is there for a lock or a page that
 * does. An unchanged size keeps the same object, so nothing downstream re-renders.
 */
function useViewport() {
    const [size, setSize] = useState(readViewport);
    useEffect(() => {
        const read = () => {
            const next = readViewport();
            setSize((prev) => (prev.width === next.width && prev.height === next.height ? prev : next));
        };
        read();
        window.addEventListener("resize", read);
        const observer = new ResizeObserver(read);
        observer.observe(document.documentElement);
        return () => {
            window.removeEventListener("resize", read);
            observer.disconnect();
        };
    }, []);
    return size;
}

/**
 * The owner's background editor: one full-screen modal dialog. The top bar carries the
 * Desktop / Phone toggle, Cancel and Save. Under it the real header is previewed
 * life-size at the chosen layout and is where the art is positioned (drag, wheel, pinch,
 * arrow keys); the zoom, Fit, Reset and Remove sit under the preview; every art to pick
 * from fills the rest.
 *
 * The page's own header is not touched while the editor is open: the draft shows in the
 * preview alone, which can also lay the header out as on a phone, which the page behind
 * could not. Closing with unsaved changes (Cancel, Esc) asks first, in an in-app dialog.
 */
export function BackgroundEditor({ profile, saved, onClose }: IBackgroundEditorProps) {
    const t: TypedT<typeof messages> = useT("user");
    const viewport = useViewport();
    const [mode, setMode] = useState<PreviewMode>(() => defaultPreviewMode(viewport.width));
    const { draft, dirty, saving, pick, adjust, remove, save } = useBackgroundDraft(saved, onClose);
    const [confirming, setConfirming] = useState(false);
    // Focus lands on the body on open, not on the first control, so no focus ring greets the author; Tab goes on from there.
    const bodyRef = useRef<HTMLDivElement>(null);
    // State, not refs: the dialog's portal mounts its content after this component's effects run, so the strip's listeners attach once the elements exist.
    const [body, setBody] = useState<HTMLDivElement | null>(null);
    const [preview, setPreview] = useState<HTMLElement | null>(null);
    const collapsed = useStrip(body, preview);
    const bodyRefs = useCallback((node: HTMLDivElement | null) => {
        bodyRef.current = node;
        setBody(node);
    }, []);

    const [deadAxis, showDeadAxis] = useDeadAxis();

    const viewportWidth = previewViewport(mode, viewport.width);
    // The browser sees the picked art by kind and id alone, the same object while those hold, so a drag's every move skips it.
    const pickedKind = draft?.kind;
    const pickedId = draft?.id;
    const selected = useMemo<IPickedArt | null>(() => (pickedKind !== undefined && pickedId !== undefined ? { kind: pickedKind, id: pickedId } : null), [pickedKind, pickedId]);
    // The hidden strip keeps the crop it last showed: it is out of sight and takes no input, so a drag on the full preview does not re-render its frame each move. A new art (pick, elite, removal) still reaches it, so the art is loaded before it shows; the live crop does the moment it shows.
    const stripHeld = useRef(draft);
    if (collapsed || !sameArt(stripHeld.current, draft)) stripHeld.current = draft;
    const stripDraft = collapsed ? draft : stripHeld.current;
    const expand = useCallback(() => bodyRef.current?.scrollTo({ top: 0, behavior: "smooth" }), []);

    const requestClose = () => {
        if (saving) return;
        if (dirty) setConfirming(true);
        else onClose();
    };

    return (
        <Dialog open onOpenChange={(open) => !open && requestClose()}>
            <DialogPortal>
                <DialogPrimitive.Popup
                    initialFocus={bodyRef}
                    className="fixed inset-0 z-50 flex flex-col bg-background text-foreground outline-none transition-[opacity,translate] duration-200 ease-out data-ending-style:translate-y-2 data-starting-style:translate-y-2 data-ending-style:opacity-0 data-starting-style:opacity-0 motion-reduce:transition-none"
                >
                    <header className="flex h-14 shrink-0 items-center gap-3 border-b px-4 sm:px-6">
                        <DialogTitle className="min-w-0 truncate font-semibold text-base sm:text-lg">{t("profile.background.title")}</DialogTitle>
                        <DialogDescription className="sr-only">{t("profile.background.description")}</DialogDescription>
                        <div className="ms-auto flex shrink-0 items-center gap-2">
                            <ToggleGroup value={[mode]} onValueChange={(next) => next[0] && setMode(next[0] === "phone" ? "phone" : "desktop")} variant="outline" size="sm" aria-label={t("profile.background.mode")}>
                                <ToggleGroupItem className={SEGMENT_ITEM} value="desktop" aria-label={t("profile.background.mode.desktop")}>
                                    <MonitorIcon />
                                    <span className="max-sm:sr-only">{t("profile.background.mode.desktop")}</span>
                                </ToggleGroupItem>
                                <ToggleGroupItem className={SEGMENT_ITEM} value="phone" aria-label={t("profile.background.mode.phone")}>
                                    <SmartphoneIcon />
                                    <span className="max-sm:sr-only">{t("profile.background.mode.phone")}</span>
                                </ToggleGroupItem>
                            </ToggleGroup>
                            <Button type="button" variant="ghost" size="sm" onClick={requestClose} disabled={saving}>
                                {t("profile.background.cancel")}
                            </Button>
                            <Button type="button" size="sm" onClick={save} disabled={!dirty || saving}>
                                {saving && <Spinner />}
                                {t("profile.background.save")}
                            </Button>
                        </div>
                    </header>
                    <div className="relative flex min-h-0 flex-1 flex-col">
                        {/* One scroller for the whole body, so it scrolls down as a page does. `--strip` is how far the sticky rail and search row sit below its top: the strip's height, shown or not, so showing it changes nothing in this flow (see `stripShown`). */}
                        <div ref={bodyRefs} tabIndex={-1} style={{ "--strip": `${STRIP_HEIGHT}px` } as CSSProperties} className="relative min-h-0 flex-1 overflow-y-auto overscroll-contain outline-none">
                            <section ref={setPreview} className="flex flex-col gap-3 bg-muted/40 px-4 pt-4 pb-3 sm:px-6 sm:pt-5">
                                <PreviewStage profile={profile} background={draft} viewportWidth={viewportWidth} maxHeight={previewMaxHeight(viewport.height)} onChange={adjust} onDeadAxis={showDeadAxis} />
                                <AdjustBar draft={draft} onChange={adjust} onRemove={remove} deadAxis={deadAxis} disabled={saving} />
                            </section>
                            <ArtBrowser selected={selected} onPick={pick} />
                            <div aria-hidden="true" className="pointer-events-none sticky bottom-0 -mt-10 h-10 bg-linear-to-t from-background to-transparent" />
                        </div>
                        <CollapsedStrip shown={collapsed} profile={profile} background={stripDraft} viewportWidth={viewportWidth} onChange={adjust} onRemove={remove} onDeadAxis={showDeadAxis} onExpand={expand} disabled={saving} expandLabel={t("profile.background.expand")} />
                    </div>
                    <DiscardDialog open={confirming} onOpenChange={setConfirming} onDiscard={onClose} />
                </DialogPrimitive.Popup>
            </DialogPortal>
        </Dialog>
    );
}

/** Whether two drafts draw the same art (kind, id and elite), whatever their crop and zoom. */
const sameArt = (a: ProfileBackground | null, b: ProfileBackground | null) => a === b || (a !== null && b !== null && a.kind === b.kind && a.id === b.id && a.elite === b.elite);

interface ICollapsedStripProps {
    shown: boolean;
    profile: IUserProfile;
    background: ProfileBackground | null;
    viewportWidth: number;
    onChange: (next: ProfileBackground) => void;
    onRemove: () => void;
    onDeadAxis: (axis: "x" | "y") => void;
    onExpand: () => void;
    disabled: boolean;
    expandLabel: string;
}

/**
 * The preview, compact, while the tiles have the screen: live and draggable, with the zoom.
 * Mounted from the start so it is measured before it shows; it overlays the scroller, and
 * the scroller's flow does not depend on it (`--strip` is constant), so showing it moves
 * nothing. Memoized: while hidden it is handed the draft it last showed, so it renders
 * only when it shows, hides, or the draft changes while it shows.
 */
const CollapsedStrip = memo(function CollapsedStrip({ shown, profile, background, viewportWidth, onChange, onRemove, onDeadAxis, onExpand, disabled, expandLabel }: ICollapsedStripProps) {
    return (
        <div
            inert={!shown}
            aria-hidden={!shown}
            className={cn(
                "absolute inset-x-0 top-0 z-30 flex items-center gap-4 overflow-hidden border-b bg-background/85 px-4 backdrop-blur-md backdrop-saturate-150 sm:px-6",
                // It eases in and leaves at once, so it never ghosts over the full preview scrolling back.
                shown ? "opacity-100 transition-[opacity,translate] duration-150 motion-reduce:transition-none" : "pointer-events-none -translate-y-2 opacity-0",
            )}
            style={{ height: STRIP_HEIGHT }}
        >
            <div className="min-w-0 flex-1">
                <PreviewStage profile={profile} background={background} viewportWidth={viewportWidth} maxHeight={STRIP_PREVIEW_MAX} onChange={onChange} onDeadAxis={onDeadAxis} />
            </div>
            <div className="flex w-56 shrink-0 flex-col items-stretch gap-2 max-md:w-40">
                {/* The compact bar draws the zoom alone, never the dead-axis line, so it is not handed one. */}
                <AdjustBar draft={background} onChange={onChange} onRemove={onRemove} deadAxis={null} disabled={disabled} compact />
                <Button type="button" variant="outline" size="sm" onClick={onExpand}>
                    <ChevronsDownIcon />
                    {expandLabel}
                </Button>
            </div>
        </div>
    );
});

/** The dead axis the author last tried to move along, shown for {@link DEAD_AXIS_MS}, and the function that shows one. */
function useDeadAxis(): ["x" | "y" | null, (axis: "x" | "y") => void] {
    const [deadAxis, setDeadAxis] = useState<"x" | "y" | null>(null);
    const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
    const show = useCallback((axis: "x" | "y") => {
        setDeadAxis(axis);
        clearTimeout(timer.current);
        timer.current = setTimeout(() => setDeadAxis(null), DEAD_AXIS_MS);
    }, []);
    useEffect(() => () => clearTimeout(timer.current), []);
    return [deadAxis, show];
}

/** Asks before closing over unsaved changes: Keep editing, or Discard. */
function DiscardDialog({ open, onOpenChange, onDiscard }: { open: boolean; onOpenChange: (open: boolean) => void; onDiscard: () => void }) {
    const t: TypedT<typeof messages> = useT("user");
    return (
        <AlertDialog open={open} onOpenChange={onOpenChange}>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <AlertDialogTitle>{t("profile.background.discard.title")}</AlertDialogTitle>
                    <AlertDialogDescription>{t("profile.background.discard.description")}</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" />}>{t("profile.background.discard.keep")}</AlertDialogClose>
                    <Button type="button" variant="destructive" onClick={onDiscard}>
                        {t("profile.background.discard.confirm")}
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    );
}

/** Whether the collapsed strip shows: kept in step with the body's scroll and size (see `stripShown`). */
function useStrip(body: HTMLElement | null, preview: HTMLElement | null): boolean {
    const [shown, setShown] = useState(false);
    useEffect(() => {
        if (!body || !preview) return;
        const read = () => setShown(stripShown(preview.getBoundingClientRect().bottom - body.getBoundingClientRect().top));
        read();
        body.addEventListener("scroll", read, { passive: true });
        const observer = new ResizeObserver(read);
        observer.observe(body);
        observer.observe(preview);
        return () => {
            body.removeEventListener("scroll", read);
            observer.disconnect();
        };
    }, [body, preview]);
    return shown;
}
