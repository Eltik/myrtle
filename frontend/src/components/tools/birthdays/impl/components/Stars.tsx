import type * as React from "react";
import { rarityVar } from "../helpers";

export function Stars({ rarity }: { rarity: number }): React.ReactElement {
    return (
        <span className="font-sans font-semibold text-[13px] leading-none tracking-[-1px]" style={{ color: rarityVar(rarity) }}>
            {"★".repeat(rarity)}
        </span>
    );
}
