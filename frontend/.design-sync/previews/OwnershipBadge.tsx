import { OwnershipBadge } from "frontend";

// RARITY_HEX from src/lib/utils.ts — the accent the operator cards pass in.
const RARITY = { 6: "#f7a452", 5: "#f7e79e", 4: "#bcabdb", 3: "#88c8e3", 2: "#7ef2a3", 1: "#ffffff" };

export const ShareRange = () => (
    <div className="w-fit rounded-xl border border-border bg-card p-4">
        <div className="mb-3 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">Ownership across 8,412 imported doctors</div>
        <div className="flex flex-col gap-2.5">
            {[
                { name: "Myrtle", rarity: 4, owners: 7984, pct: 0.9491 },
                { name: "Texas", rarity: 5, owners: 6120, pct: 0.7275 },
                { name: "Młynar", rarity: 6, owners: 2914, pct: 0.3464 },
                { name: "Wiš'adel", rarity: 6, owners: 731, pct: 0.0869 },
                { name: "Shu", rarity: 6, owners: 402, pct: 0.0478 },
                { name: "Durin", rarity: 2, owners: 118, pct: 0.014 },
            ].map((row) => (
                <div key={row.name} className="flex items-center gap-3">
                    <span className="w-24 text-foreground text-sm">{row.name}</span>
                    <OwnershipBadge info={{ owners: row.owners, pct: row.pct }} color={RARITY[row.rarity]} />
                    <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{row.owners.toLocaleString()} owners</span>
                </div>
            ))}
        </div>
    </div>
);

export const OnPortraitTile = () => (
    <div className="flex gap-2">
        {[
            { id: "char_4064_mlynar", name: "Młynar", rarity: 6, owners: 2914, pct: 0.3464 },
            { id: "char_102_texas", name: "Texas", rarity: 5, owners: 6120, pct: 0.7275 },
            { id: "char_151_myrtle", name: "Myrtle", rarity: 4, owners: 7984, pct: 0.9491 },
            { id: "char_124_kroos", name: "Kroos", rarity: 3, owners: 8102, pct: 0.9631 },
        ].map((op) => (
            <div key={op.id} className="w-24 rounded bg-card p-1.5">
                <div className="truncate text-[11px] text-foreground leading-tight">{op.name}</div>
                <div className="relative mt-1 aspect-square overflow-hidden" style={{ borderBottom: `4px solid ${RARITY[op.rarity]}` }}>
                    <OwnershipBadge info={{ owners: op.owners, pct: op.pct }} color={RARITY[op.rarity]} className="absolute top-0.5 left-0.5 z-10" />
                    <img alt={op.name} className="h-full w-full object-contain" src={`https://api.myrtle.moe/api/avatar/${op.id}`} />
                </div>
            </div>
        ))}
    </div>
);

export const InListRow = () => (
    <div className="flex w-full max-w-2xl flex-col gap-1">
        {[
            { id: "char_263_skadi", name: "Skadi", cls: "Guard", rarity: 6, owners: 4408, pct: 0.5239 },
            { id: "char_128_plosis", name: "Ptilopsis", cls: "Medic", rarity: 5, owners: 7215, pct: 0.8577 },
            { id: "char_187_ccheal", name: "Gavial", cls: "Medic", rarity: 4, owners: 7761, pct: 0.9226 },
        ].map((op) => (
            <div key={op.id} className="flex items-center gap-3 rounded-lg border border-transparent bg-card/50 px-3 py-2.5">
                <img alt={op.name} className="h-12 w-12 rounded-md border border-border/50 bg-background object-cover" src={`https://api.myrtle.moe/api/avatar/${op.id}`} />
                <span className="flex-1 font-semibold text-foreground text-sm uppercase tracking-wide">{op.name}</span>
                <span className="w-20 text-muted-foreground text-sm">{op.cls}</span>
                <OwnershipBadge info={{ owners: op.owners, pct: op.pct }} color={RARITY[op.rarity]} />
            </div>
        ))}
    </div>
);

export const UnknownPopulation = () => (
    <div className="w-fit rounded-xl border border-border bg-card p-4">
        <div className="mb-3 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">Denominator unavailable</div>
        <div className="flex items-center gap-3">
            <span className="w-24 text-foreground text-sm">Ines</span>
            <OwnershipBadge info={{ owners: 1284, pct: null }} color={RARITY[6]} />
            <span className="text-[11px] text-muted-foreground">1,284 owners recorded, but the player-population total has not synced — the share falls back to 0%</span>
        </div>
    </div>
);
