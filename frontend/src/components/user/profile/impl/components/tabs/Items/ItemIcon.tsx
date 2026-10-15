import { useState } from "react";
import { env } from "#/env";
import { RARITY_COLORS } from "./helpers";
import type { IItemEntry } from "./types";

interface IItemIconProps {
    item: IItemEntry;
    size?: number;
    className?: string;
}

/**
 * `null` while the entry's icon is not known yet (the item table is still
 * loading), so nothing is requested: an inventory id such as `30043` is never
 * a file stem, the icon is `MTL_SL_IRON3`.
 */
export function itemIconURL(iconId: string | null): string | null {
    return iconId ? `${env.VITE_BACKEND_URL ?? ""}/api/item-icon/${encodeURIComponent(iconId)}` : null;
}

export function ItemIcon({ item, size = 64, className = "" }: IItemIconProps) {
    // The URL that failed, not a flag: when the item table arrives and the
    // entry's icon changes, the new URL gets its own attempt.
    const [failedURL, setFailedURL] = useState<string | null>(null);
    const color = RARITY_COLORS[item.rarityNum] ?? "#b5b5b5";
    const url = itemIconURL(item.iconId);
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
            {url && url !== failedURL ? (
                <img alt={item.name} src={url} className="h-full w-full object-contain" decoding="async" loading="lazy" onError={() => setFailedURL(url)} />
            ) : (
                <div className="flex h-full w-full items-center justify-center font-bold font-mono tabular-nums" style={{ color, fontSize: size * 0.32, letterSpacing: "-0.02em" }}>
                    <span className="drop-shadow-[0_1px_0_rgb(0_0_0/0.4)]">{initials}</span>
                </div>
            )}
        </div>
    );
}
