import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { cn } from "#/lib/utils";

/**
 * Square operator portrait with a rarity-coloured bottom edge. The edge is an
 * overlay so the portrait image cannot paint over it.
 */
export function RarityTile({ id, name, rarity, size }: { id: string; name: string; rarity: number; size: "sm" | "lg" }): React.ReactElement {
    return (
        <span
            className={cn(
                "relative grid shrink-0 place-items-center overflow-hidden bg-muted font-semibold after:pointer-events-none after:absolute after:inset-0 after:rounded-[inherit]",
                size === "sm" ? "size-[34px] rounded-lg text-[13px] after:shadow-[inset_0_-2px_0_var(--tile-rarity)]" : "size-11 rounded-[10px] text-[16px] after:shadow-[inset_0_-3px_0_var(--tile-rarity)]",
            )}
            style={{ "--tile-rarity": `var(--rarity-${rarity})` } as React.CSSProperties}
        >
            <OperatorAvatar charId={id} name={name} />
        </span>
    );
}
