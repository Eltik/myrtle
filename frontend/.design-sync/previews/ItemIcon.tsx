import { ItemIcon } from "frontend";

const item = (item_id: string, name: string, rarityNum: number, iconId: string | null, quantity: number, category = "mat") => ({
    item_id,
    quantity,
    name,
    rarityNum,
    category,
    iconId,
    expValue: null,
    meta: null,
});

const RARITY_LADDER = [
    item("30011", "Orirock", 1, "MTL_SL_G1", 2140),
    item("30012", "Orirock Cube", 2, "MTL_SL_G2", 1284),
    item("30013", "Orirock Cluster", 3, "MTL_SL_G3", 412),
    item("30084", "Manganese Trihydrate", 4, "MTL_SL_MANGANESE2", 74),
    item("30115", "Polymerization Preparation", 5, "MTL_SL_PP", 18),
];

const ORIROCK_CLUSTER = RARITY_LADDER[2];
const MISSING_ART = item("mod_unlock_token_unreleased", "Module Data Block", 5, "mod_unlock_token_unreleased", 3, "module");

export const RarityRange = () => (
    <div className="flex flex-wrap items-end gap-4">
        {RARITY_LADDER.map((it) => (
            <div className="flex w-24 flex-col items-center gap-2" key={it.item_id}>
                <ItemIcon item={it} size={64} />
                <span className="text-center font-mono text-[10.5px] text-muted-foreground uppercase leading-tight tracking-[0.12em]">{it.rarityNum}★</span>
            </div>
        ))}
    </div>
);

export const SizeScale = () => (
    <div className="flex items-end gap-5">
        {[32, 48, 64, 96].map((size) => (
            <div className="flex flex-col items-center gap-2" key={size}>
                <ItemIcon item={ORIROCK_CLUSTER} size={size} />
                <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums leading-none">{size}px</span>
            </div>
        ))}
    </div>
);

export const InInventoryRow = () => (
    <div className="flex w-full max-w-md flex-col divide-y divide-border/60 overflow-hidden rounded-xl border border-border bg-card">
        {RARITY_LADDER.slice(1).map((it) => (
            <div className="flex items-center gap-3 px-4 py-2.5" key={it.item_id}>
                <ItemIcon item={it} size={40} />
                <span className="min-w-0 flex-1 truncate font-sans text-[13.5px] text-foreground leading-tight">{it.name}</span>
                <span className="font-mono font-semibold text-[13px] text-foreground tabular-nums">{it.quantity.toLocaleString()}</span>
            </div>
        ))}
    </div>
);

export const FallbackInitials = () => (
    <div className="flex items-center gap-4">
        <ItemIcon item={MISSING_ART} size={96} />
        <div className="flex max-w-xs flex-col gap-1">
            <span className="font-sans font-semibold text-[13.5px] text-foreground leading-tight tracking-tight">Module Data Block</span>
            <span className="font-mono text-[11px] text-muted-foreground leading-snug">When the icon asset is missing, the tile falls back to rarity-tinted initials.</span>
        </div>
    </div>
);
