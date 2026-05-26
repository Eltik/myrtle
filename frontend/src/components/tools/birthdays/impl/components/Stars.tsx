import type * as React from "react";
import { rarityVar } from "../helpers";

/** Rarity rendered as filled stars in the tier colour (e.g. 6★ = six coral-gold stars). */
export function Stars({ rarity }: { rarity: number }): React.ReactElement {
    return (
        <span className="font-sans font-semibold text-[13px] leading-none tracking-[-1px]" style={{ color: rarityVar(rarity) }}>
            {"★".repeat(rarity)}
        </span>
    );
}
