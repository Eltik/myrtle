import { SkinPopupProvider, SkinTileView } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// The release planner's skin popup host: any SkinTileView or SkinCard under it
// opens the outfit's detail dialog. The popup reads the skins index through
// server functions, which reject in the design bundle, so the opened dialog shows
// its real "not found" branch with the outfit's name.
const LOOKUP = new Map();

function ClickOnMount({ children }: { children: ReactNode }) {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let f2 = 0;
        const timers: number[] = [];
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => {
                ref.current?.querySelector<HTMLButtonElement>("button")?.click();
                for (const ms of [60, 180, 400]) timers.push(window.setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
            });
        });
        return () => {
            cancelAnimationFrame(f1);
            cancelAnimationFrame(f2);
            for (const t of timers) window.clearTimeout(t);
        };
    }, []);
    return (
        <div ref={ref} className="relative w-full p-4" style={{ minHeight: 520 }}>
            {children}
        </div>
    );
}

const Tiles = () => (
    <div className="flex gap-2">
        <SkinTileView skinId="char_249_mlyss@boc#8" charId="char_249_mlyss" charName={{ text: "Muelsyse", source: "memory" }} skinName="Young Branch" skinNameEn="Young Branch" portraitPath="/en/skin-portrait/char_249_mlyss_boc#8" lookup={LOOKUP} />
        <SkinTileView skinId="char_293_thorns@boc#8" charId="char_293_thorns" charName={{ text: "Thorns", source: "memory" }} skinName="Blade-cleaved Tides" skinNameEn="Blade-cleaved Tides" portraitPath="/en/skin-portrait/char_293_thorns_boc#8" lookup={LOOKUP} />
    </div>
);

// Closed: the provider is invisible, its tiles render as normal.
export const Closed = () => (
    <div className="w-full p-4">
        <SkinPopupProvider>
            <Tiles />
        </SkinPopupProvider>
    </div>
);

// A tile clicked: the dialog opens over the page.
export const Opened = () => (
    <ClickOnMount>
        <SkinPopupProvider>
            <Tiles />
        </SkinPopupProvider>
    </ClickOnMount>
);
