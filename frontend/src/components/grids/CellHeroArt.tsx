import { useMemo, useState } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { EntityAvatar, entityShape } from "#/components/tier-lists/entities";
import type { ITierEntity } from "#/lib/api/tier-entities";
import { useGamedataServer } from "#/lib/i18n";
import { cn } from "#/lib/utils";
import { heroArtPaths } from "./cellViewer";

// A picked cell's art drawn large, for the full-screen viewer and the phone
// cell editor. An operator or skin tries its full art (see `heroArtPaths`)
// with the tile art showing until it loads, and keeps the tile art if none
// loads; every other kind draws its tile art in a large frame.

interface ICellHeroArtProps {
    entity: ITierEntity;
    /** Where the pick's art lives when it is not the reader's server (`cn` for a CN-only operator). */
    server: string | null;
    /** `viewer` fills a full screen's stage; `sheet` is the editor's smaller preview. */
    size: "viewer" | "sheet";
}

/**
 * The tile art's frame. The viewer's caps keep it at about twice the art's own
 * size (an avatar is 180 px, an event banner 280 px wide), past which it blurs.
 */
const TILE_SIZE = {
    viewer: { square: "w-[min(72vw,44dvh,360px)]", wide: "w-[min(92vw,560px)]" },
    sheet: { square: "w-[min(56vw,26dvh,240px)]", wide: "w-[min(88vw,480px)]" },
} as const;

export function CellHeroArt({ entity, server, size }: ICellHeroArtProps) {
    // Keyed by the entity, so stepping to another cell starts its own chain of tries.
    return <HeroChain key={entity.key} entity={entity} server={server} size={size} />;
}

function HeroChain({ entity, server, size }: ICellHeroArtProps) {
    const reader = useGamedataServer();
    const artServer = server ?? reader;
    const paths = useMemo(() => heroArtPaths(entity), [entity]);
    const [attempt, setAttempt] = useState(0);
    const [loaded, setLoaded] = useState(false);
    const path = paths[attempt];
    const src = path ? asset(path, artServer === "cn" ? "cn" : undefined) : null;
    const wide = entityShape(entity) === "wide";

    return (
        <div className="relative flex h-full w-full items-center justify-center">
            {!loaded && (
                <span
                    className={cn(
                        "relative flex items-center justify-center overflow-hidden rounded-xl bg-[oklch(0.24_0.005_285)] text-5xl text-white shadow-[0_18px_48px_-18px_oklch(0_0_0/0.7)] ring-1 ring-white/8",
                        wide ? "aspect-[2.1/1]" : "aspect-square",
                        TILE_SIZE[size][wide ? "wide" : "square"],
                        src && "opacity-60",
                    )}
                    aria-hidden="true"
                >
                    <EntityAvatar entity={entity} face="tile" tone="dark" server={server ?? undefined} />
                </span>
            )}
            {src && (
                <img
                    src={src}
                    alt=""
                    aria-hidden="true"
                    decoding="async"
                    draggable={false}
                    onLoad={() => setLoaded(true)}
                    onError={() => {
                        setLoaded(false);
                        setAttempt((n) => n + 1);
                    }}
                    className={cn("pointer-events-none absolute inset-0 block h-full w-full select-none object-contain transition-opacity duration-200 motion-reduce:transition-none", loaded ? "opacity-100" : "opacity-0")}
                />
            )}
        </div>
    );
}
