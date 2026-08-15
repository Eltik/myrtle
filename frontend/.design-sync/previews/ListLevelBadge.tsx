import { ListLevelBadge } from "frontend";

// Threat tier is the enemy database's primary colour axis: ELITE is amber
// (--enemy-elite), BOSS is brand red, and NORMAL deliberately renders nothing —
// the list view substitutes plain "Normal" text instead (see EnemyCardList).
const PORTRAIT = "https://api.myrtle.moe/api/assets/textures/spritepack/icon_enemies_2/enemy_1500_skulsr.png";

export const ThreatTiers = () => (
    <div className="flex flex-col gap-3">
        <div className="flex items-center gap-2.5">
            <ListLevelBadge level="ELITE" />
            <span className="font-sans font-semibold text-[13px] text-foreground uppercase tracking-[0.03em]">Sarkaz Crossbowman</span>
            <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.14em]">S3</span>
        </div>
        <div className="flex items-center gap-2.5">
            <ListLevelBadge level="BOSS" />
            <span className="font-sans font-semibold text-[13px] text-foreground uppercase tracking-[0.03em]">Skullshatterer</span>
            <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.14em]">SS</span>
        </div>
        <div className="flex items-center gap-2.5">
            <span className="font-medium font-sans text-[11.5px] text-muted-foreground">Normal</span>
            <span className="font-sans font-semibold text-[13px] text-foreground uppercase tracking-[0.03em]">Originium Slug</span>
            <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.14em]">B1</span>
        </div>
    </div>
);

export const CompactOnPortrait = () => (
    <div className="relative w-40 overflow-hidden rounded-md border border-border bg-card">
        <div className="relative aspect-square w-full bg-muted/40">
            <img src={PORTRAIT} alt="Skullshatterer portrait" className="block h-full w-full object-contain" />
            <span className="absolute top-2 left-2 rounded-sm bg-background/75 px-1.5 py-0.5 font-medium font-mono text-[9px] text-muted-foreground uppercase leading-none tracking-[0.14em]">SS</span>
            <span className="absolute top-2 right-2">
                <ListLevelBadge level="BOSS" size="sm" />
            </span>
        </div>
        <div className="px-3 py-2.5">
            <h3 className="m-0 font-sans font-semibold text-[12.5px] text-foreground uppercase leading-[1.15] tracking-[0.02em]">Skullshatterer</h3>
        </div>
    </div>
);

export const SizeScale = () => (
    <div className="flex flex-col gap-4">
        <div className="flex items-center gap-3">
            <span className="w-16 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.14em]">Default</span>
            <ListLevelBadge level="ELITE" />
            <ListLevelBadge level="BOSS" />
        </div>
        <div className="flex items-center gap-3">
            <span className="w-16 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.14em]">Small</span>
            <ListLevelBadge level="ELITE" size="sm" />
            <ListLevelBadge level="BOSS" size="sm" />
        </div>
    </div>
);
