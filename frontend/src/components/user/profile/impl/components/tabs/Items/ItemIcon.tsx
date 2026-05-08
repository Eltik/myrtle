import { useState } from "react";
import { env } from "#/env";
import { RARITY_COLORS } from "./helpers";
import type { IItemEntry } from "./types";

interface IItemIconProps {
    item: IItemEntry;
    size?: number;
    className?: string;
}

function itemIconUrl(iconId: string | null, fallbackId: string): string {
    return `${env.VITE_BACKEND_URL ?? ""}/api/item-icon/${iconId ?? fallbackId}`;
}

export function ItemIcon({ item, size = 64, className = "" }: IItemIconProps) {
    const [errored, setErrored] = useState(false);
    const color = RARITY_COLORS[item.rarityNum] ?? "#b5b5b5";
    const url = itemIconUrl(item.iconId, item.item_id);
    const initials =
        item.name
            .replace(/[--]/g, " ")
            .split(" ")
            .filter(Boolean)
            .slice(0, 2)
            .map((w) => w[0])
            .join("")
            .toUpperCase() || "?";

    const baseStyle: React.CSSProperties = {
        width: size,
        height: size,
        background: `radial-gradient(ellipse at 30% 30%, ${color}33, transparent 70%), linear-gradient(135deg, ${color}1a, color-mix(in srgb, ${color} 8%, var(--muted)) 80%)`,
    };

    return (
        <div className={`relative aspect-square shrink-0 overflow-hidden rounded-md ${className}`} style={baseStyle}>
            {!errored ? (
                <img alt={item.name} src={url} className="h-full w-full object-contain" decoding="async" loading="lazy" onError={() => setErrored(true)} />
            ) : (
                <div className="flex h-full w-full items-center justify-center font-mono font-bold tabular-nums" style={{ color, fontSize: size * 0.32, letterSpacing: "-0.02em" }}>
                    <span className="drop-shadow-[0_1px_0_rgb(0_0_0/0.4)]">{initials}</span>
                </div>
            )}
        </div>
    );
}
