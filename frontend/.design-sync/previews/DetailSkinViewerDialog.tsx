import { DetailSkinViewerDialog } from "frontend";
import { Maximize2 } from "lucide-react";
import { type ReactNode, useEffect, useRef } from "react";

const MLYNAR_E2 = "https://api.myrtle.moe/api/assets/textures/chararts/char_4064_mlynar/char_4064_mlynar_2.png";
const SKADI_E2 = "https://api.myrtle.moe/api/assets/textures/chararts/char_263_skadi/char_263_skadi_2.png";

/** The fullscreen button as `SkinsContent` composes it — pinned over the skin art. */
const FullscreenButton = () => (
    <button type="button" aria-label="Fullscreen" className="absolute top-3 right-3 inline-flex items-center justify-center rounded-md border border-white/20 bg-black/40 p-1.5 text-white backdrop-blur-md transition-colors hover:bg-black/60">
        <Maximize2 className="h-4 w-4" />
    </button>
);

/** The art frame the trigger lives in, straight out of `SkinsContent`. */
const ArtFrame = ({ image, kicker, name, artist, children }: { image: string; kicker: string; name: string; artist: string; children: ReactNode }) => (
    <div className="relative aspect-4/3 w-full overflow-hidden rounded-xl border border-border bg-linear-to-b from-secondary/30 via-secondary/10 to-secondary/40">
        <img alt={name} className="absolute inset-0 h-full w-full object-contain" src={image} />
        <div className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-linear-to-t from-black/70 via-black/40 to-transparent" />
        {children}
        <div className="absolute inset-x-0 bottom-0 p-5">
            <div className="font-mono text-[10px] text-white/70 uppercase tracking-wider">{kicker}</div>
            <div className="mt-0.5 font-semibold text-2xl text-white drop-shadow-md">{name}</div>
            <div className="mt-0.5 text-white/70 text-xs">Artist · {artist}</div>
        </div>
    </div>
);

/** Base UI wires the dialog trigger after first paint, so the click that opens it
 *  has to wait two frames. */
const AutoOpen = ({ children }: { children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => ref.current?.querySelector("button")?.click());
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, []);
    return (
        <div className="relative min-h-[520px] w-full" ref={ref}>
            {children}
        </div>
    );
};

export const Opened = () => (
    <AutoOpen>
        <DetailSkinViewerDialog imageSrc={MLYNAR_E2} skinName="Młynar — Elite 2">
            <button type="button" aria-label="Fullscreen" className="inline-flex items-center gap-2 rounded-md border border-border bg-card px-3 py-1.5 text-foreground text-sm">
                <Maximize2 className="h-4 w-4" />
                View Elite 2 art
            </button>
        </DetailSkinViewerDialog>
    </AutoOpen>
);

export const TriggerOnSkinArt = () => (
    <div className="w-full max-w-2xl">
        <ArtFrame image={MLYNAR_E2} kicker="Elite 2" name="Młynar" artist="竜崎いち">
            <DetailSkinViewerDialog imageSrc={MLYNAR_E2} skinName="Młynar — Elite 2">
                <FullscreenButton />
            </DetailSkinViewerDialog>
        </ArtFrame>
    </div>
);

export const AlternateOutfitTrigger = () => (
    <div className="w-full max-w-2xl">
        <ArtFrame image={SKADI_E2} kicker="Elite 2" name="Skadi" artist="alchemaniac">
            <DetailSkinViewerDialog imageSrc={SKADI_E2} skinName="Skadi — Elite 2">
                <FullscreenButton />
            </DetailSkinViewerDialog>
        </ArtFrame>
    </div>
);
