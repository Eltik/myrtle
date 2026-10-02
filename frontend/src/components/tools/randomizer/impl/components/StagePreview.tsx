import { Maximize2 } from "lucide-react";
import * as React from "react";
import { memo, useCallback, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Dialog, DialogContent, DialogTitle, DialogTrigger } from "#/components/ui/dialog";
import { stagePreviewURLs } from "#/lib/api/stages";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { IStage } from "#/types/stages";
import type { messages } from "./StagePreview.messages";

type PreviewT = TypedT<typeof messages>;

interface IStagePreviewProps {
    stage: IStage;
    className?: string;
}

const RESOLVED_PREVIEW = new Map<string, string>();
const FAILED_PREVIEW = new Set<string>();

export function StagePreview({ stage, className }: IStagePreviewProps): React.ReactElement {
    const t: PreviewT = useT("tools");
    const urls = React.useMemo(() => stagePreviewURLs(stage), [stage]);
    const cached = RESOLVED_PREVIEW.get(stage.stageId) ?? null;
    const [resolved, setResolved] = useState<string | null>(cached);
    const [resolving, setResolving] = useState<boolean>(cached === null);

    useEffect(() => {
        const hit = RESOLVED_PREVIEW.get(stage.stageId);
        if (hit) {
            setResolved(hit);
            setResolving(false);
            return;
        }
        setResolved(null);
        setResolving(true);

        let cancelled = false;
        let i = 0;
        const probe = () => {
            while (i < urls.length && FAILED_PREVIEW.has(urls[i] ?? "")) i++;
            if (cancelled) return;
            if (i >= urls.length) {
                setResolving(false);
                return;
            }
            const url = urls[i] ?? "";
            const img = new Image();
            img.onload = () => {
                if (cancelled) return;
                RESOLVED_PREVIEW.set(stage.stageId, url);
                setResolved(url);
                setResolving(false);
            };
            img.onerror = () => {
                FAILED_PREVIEW.add(url);
                i++;
                probe();
            };
            img.src = url;
        };
        probe();

        return () => {
            cancelled = true;
        };
    }, [stage.stageId, urls]);

    const failed = !resolving && resolved === null;
    const canExpand = resolved !== null;
    const labelName = stage.name ? t("randomizer.preview.stageLabel", { code: stage.code, name: stage.name }) : stage.code;

    const thumbnail = (
        <button
            type="button"
            disabled={!canExpand}
            aria-label={canExpand ? t("randomizer.preview.expand", { stage: labelName }) : t("randomizer.preview.none", { code: stage.code })}
            className={cn(
                "group relative block aspect-video w-full overflow-hidden rounded-lg border border-border/60 bg-muted/40 outline-none transition-shadow",
                canExpand && "cursor-zoom-in focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
                !canExpand && "cursor-default",
                className,
            )}
        >
            {resolved && <img src={resolved} alt={t("randomizer.preview.alt", { stage: labelName })} loading="lazy" decoding="async" className="absolute inset-0 h-full w-full object-fill transition-transform duration-300 ease-out group-hover:scale-[1.02]" />}
            {failed && <PreviewFallback code={stage.code} t={t} />}
            <div aria-hidden="true" className="pointer-events-none absolute inset-x-0 bottom-0 h-12 bg-linear-to-t from-black/40 to-transparent dark:from-black/60" />
            {canExpand && (
                <span aria-hidden="true" className="pointer-events-none absolute top-2 right-2 inline-flex size-7 items-center justify-center rounded-md bg-black/55 text-white opacity-0 backdrop-blur-sm transition-opacity duration-200 group-hover:opacity-100 group-focus-visible:opacity-100">
                    <Maximize2 className="size-3.5" />
                </span>
            )}
        </button>
    );

    if (!resolved) {
        return thumbnail;
    }

    return (
        <StageViewer imageSrc={resolved} stageName={labelName}>
            {thumbnail}
        </StageViewer>
    );
}

function PreviewFallback({ code, t }: { code: string; t: PreviewT }): React.ReactElement {
    return (
        <div className="absolute inset-0 flex items-center justify-center bg-[radial-gradient(circle_at_30%_20%,var(--lagoon,#4fb8b2)/20,transparent_60%),radial-gradient(circle_at_70%_80%,var(--palm,#2f6a4a)/15,transparent_60%)] text-foreground/40">
            <svg aria-hidden="true" viewBox="0 0 64 64" className="h-10 w-10" fill="none" stroke="currentColor" strokeWidth="1.5">
                <rect x="6" y="14" width="52" height="36" rx="4" />
                <path d="M6 36 L20 28 L32 38 L46 26 L58 34" />
                <circle cx="22" cy="22" r="3" />
            </svg>
            <span className="sr-only">{t("randomizer.preview.none", { code })}</span>
        </div>
    );
}

interface IStageViewerProps {
    imageSrc: string;
    stageName: string;
    children: React.ReactElement<Record<string, unknown>>;
}

const GROW_DELAY_MS = 200;
const GROW_DURATION_MS = 200;
const GROWN_WIDTH_PX = 640;
const VIEWPORT_MARGIN_PX = 16;

interface IGrowth {
    /** Where the thumbnail sits, in viewport pixels. */
    from: DOMRect;
    left: number;
    top: number;
    width: number;
}

/** The grown box: centred on the thumbnail, pulled back inside the viewport. */
function growthFor(from: DOMRect): IGrowth | null {
    const width = Math.min(GROWN_WIDTH_PX, window.innerWidth - 2 * VIEWPORT_MARGIN_PX);
    if (width <= from.width) return null;
    const height = (width * 9) / 16;
    const clamp = (v: number, max: number) => Math.min(Math.max(v, VIEWPORT_MARGIN_PX), max - VIEWPORT_MARGIN_PX);
    return {
        from,
        left: clamp(from.left + from.width / 2 - width / 2, window.innerWidth - width),
        top: clamp(from.top + from.height / 2 - height / 2, window.innerHeight - height),
        width,
    };
}

/**
 * The thumbnail grows in place on hover, and opens the map on its own on
 * click. The grown copy is a fixed layer over the page rather than a scale on
 * the thumbnail itself, because the slab clips its overflow. No zoom or pan:
 * the source is a 512 px square, so past roughly twice the thumbnail there is
 * no more detail to find, only blur.
 */
const StageViewer = memo(function StageViewer({ imageSrc, stageName, children }: IStageViewerProps) {
    const [viewing, setViewing] = useState(false);
    const [growth, setGrowth] = useState<IGrowth | null>(null);
    const [grown, setGrown] = useState(false);
    const timer = useRef<number | undefined>(undefined);

    // Unmounts on a timer rather than on `transitionend`, which never fires
    // under reduced motion.
    const shrink = useCallback(() => {
        window.clearTimeout(timer.current);
        setGrown(false);
        timer.current = window.setTimeout(() => setGrowth(null), GROW_DURATION_MS);
    }, []);

    useEffect(() => {
        if (!growth) return;
        const frame = requestAnimationFrame(() => setGrown(true));
        window.addEventListener("scroll", shrink, { capture: true, passive: true });
        window.addEventListener("resize", shrink);
        return () => {
            cancelAnimationFrame(frame);
            window.removeEventListener("scroll", shrink, { capture: true });
            window.removeEventListener("resize", shrink);
        };
    }, [growth, shrink]);

    useEffect(() => () => window.clearTimeout(timer.current), []);

    const onPointerEnter = useCallback((e: React.PointerEvent<HTMLElement>) => {
        if (e.pointerType !== "mouse") return;
        const target = e.currentTarget;
        window.clearTimeout(timer.current);
        timer.current = window.setTimeout(() => setGrowth(growthFor(target.getBoundingClientRect())), GROW_DELAY_MS);
    }, []);

    const onPointerLeave = useCallback(() => window.clearTimeout(timer.current), []);

    const height = growth ? (growth.width * 9) / 16 : 0;
    const collapsed = growth ? `translate(${growth.from.left - growth.left}px, ${growth.from.top - growth.top}px) scale(${growth.from.width / growth.width})` : undefined;

    return (
        <Dialog open={viewing} onOpenChange={setViewing}>
            <DialogTrigger render={children} onPointerEnter={onPointerEnter} onPointerLeave={onPointerLeave} />
            {growth &&
                createPortal(
                    <button
                        type="button"
                        tabIndex={-1}
                        aria-hidden="true"
                        onPointerLeave={shrink}
                        onClick={() => {
                            shrink();
                            setViewing(true);
                        }}
                        className={cn(
                            "fixed z-40 block origin-top-left cursor-zoom-in overflow-hidden rounded-lg border border-border/60 bg-muted outline-none transition-[transform,box-shadow] ease-out motion-reduce:transition-none",
                            grown ? "shadow-[0_32px_64px_-16px_rgb(0_0_0/0.45),0_12px_24px_-12px_rgb(0_0_0/0.3)]" : "pointer-events-none shadow-none",
                        )}
                        style={{ left: growth.left, top: growth.top, width: growth.width, height, transform: grown ? "none" : collapsed, transitionDuration: `${GROW_DURATION_MS}ms` }}
                    >
                        <StageMapImage src={imageSrc} alt="" />
                    </button>,
                    document.body,
                )}
            <DialogContent className="w-[min(80rem,95vw)] max-w-[min(80rem,95vw)] overflow-hidden p-0 sm:max-w-[min(80rem,95vw)]" showCloseButton bottomStickOnMobile={false}>
                <DialogTitle className="sr-only">{stageName}</DialogTitle>
                <StageMapImage src={imageSrc} alt={stageName} />
            </DialogContent>
        </Dialog>
    );
});

/**
 * Every stage map ships as a square the game stretches to 16:9 on screen, so
 * the box is fixed at 16:9 and the image fills it whatever shape the file is.
 */
function StageMapImage({ src, alt }: { src: string; alt: string }): React.ReactElement {
    return <img src={src} alt={alt} decoding="async" draggable={false} className="block aspect-video w-full object-fill" />;
}
