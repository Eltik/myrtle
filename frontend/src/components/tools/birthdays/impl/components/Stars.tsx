import type * as React from "react";
import { rarityInkVar } from "../helpers";

export function Stars({ rarity }: { rarity: number }): React.ReactElement {
    return (
        <span className="font-sans font-semibold text-[13px] leading-none tracking-[-1px]" style={{ color: rarityInkVar(rarity) }}>
            {"★".repeat(rarity)}
        </span>
    );
}
