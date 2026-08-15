import { EnemyPlaceholder } from "frontend";

// The fallback silhouette drawn wherever an enemy portrait is missing or 404s
// (EnemyCardGrid, EnemyCardList and EnemyHero all swap to it via onError).
export const GridCardFallback = () => (
    <div className="w-40 overflow-hidden rounded-md border border-border bg-card">
        <div className="relative aspect-square w-full bg-muted/40">
            <EnemyPlaceholder className="absolute inset-0 h-full w-full p-4" />
            <span className="absolute top-2 left-2 rounded-sm bg-background/75 px-1.5 py-0.5 font-medium font-mono text-[9px] text-muted-foreground uppercase leading-none tracking-[0.14em]">DUR8</span>
        </div>
        <div className="px-3 py-2.5">
            <h3 className="m-0 font-sans font-semibold text-[12.5px] text-foreground uppercase leading-[1.15] tracking-[0.02em]">Volleyball Spiking Cart</h3>
        </div>
    </div>
);

export const ListRowThumbnail = () => (
    <div className="flex w-96 items-center gap-3.5 rounded-lg border border-border bg-card px-3.5 py-2.5">
        <div className="relative h-12 w-12 shrink-0 overflow-hidden rounded-md border border-border bg-muted/40">
            <EnemyPlaceholder className="absolute inset-0 h-full w-full p-1.5" />
        </div>
        <div className="flex min-w-0 flex-col gap-1">
            <span className="truncate font-sans font-semibold text-[13px] text-foreground uppercase leading-none tracking-[0.03em]">Iron Lampstand</span>
            <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.14em]">AN32 · Apparition</span>
        </div>
    </div>
);

export const HeroFallback = () => (
    <div className="flex w-full max-w-xl gap-5 rounded-xl border border-border bg-card p-5">
        <div className="relative h-28 w-28 shrink-0 overflow-hidden rounded-lg border border-border bg-muted/40">
            <EnemyPlaceholder className="absolute inset-0 h-full w-full p-2.5" />
        </div>
        <div className="flex min-w-0 flex-1 flex-col justify-center gap-1.5">
            <h1 className="m-0 font-bold font-sans text-[26px] text-foreground leading-[1.1] tracking-tight">'Witness of the Sankta'</h1>
            <div className="font-medium font-mono text-[11px] text-muted-foreground uppercase leading-none tracking-[0.14em]">MT4 · Melee</div>
            <p className="m-0 mt-1 font-sans text-[13px] text-muted-foreground leading-normal">No portrait has been extracted for this enemy yet.</p>
        </div>
    </div>
);
