import type { ITile } from "#/lib/base/board";
import { tileLabel } from "../logic/tile-labels";

export function TileHint({ tile }: { tile: ITile }) {
    return (
        <span className="flex flex-col gap-1">
            <span className="font-semibold text-foreground">{tileLabel(tile)}</span>
            <span className="grid grid-cols-[auto_auto] gap-x-3 gap-y-0.5 text-muted-foreground">
                <span>Level</span>
                <span className="text-right font-mono text-foreground tabular-nums">{tile.level}</span>
            </span>
        </span>
    );
}
