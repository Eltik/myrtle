import type { ITile } from "#/lib/base/board";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { tileLabel } from "../logic/tile-labels";
import type { messages as labelMessages } from "../logic/tile-labels.messages";
import type { messages } from "./TileHint.messages";

export function TileHint({ tile }: { tile: ITile }) {
    /** The tile's own name comes from `tile-labels.messages.ts`. */
    const t: TypedT<typeof messages & typeof labelMessages> = useT("user");
    return (
        <span className="flex flex-col gap-1">
            <span className="font-semibold text-foreground">{tileLabel(tile, t)}</span>
            <span className="grid grid-cols-[auto_auto] gap-x-3 gap-y-0.5 text-muted-foreground">
                <span>{t("profile.base.tile.level")}</span>
                <span className="text-right font-mono text-foreground tabular-nums">{tile.level}</span>
            </span>
        </span>
    );
}
