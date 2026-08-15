import { StatBar } from "frontend";

// `max` is the per-threat-tier maximum across the enemies on screen, so a bar
// reads as "how heavy is this enemy for its tier" rather than an absolute value.
// Values >= 1000 are abbreviated (2.1k), >= 10000 rounded (80k).
const ATK_COLOR = "color-mix(in oklch, var(--foreground) 50%, transparent)";
const DEF_COLOR = "color-mix(in oklch, var(--foreground) 30%, transparent)";

export const EnemyCardStats = () => (
    <div className="w-56 rounded-md border border-border bg-card px-3 py-2.5">
        <h3 className="m-0 mb-2 font-sans font-semibold text-[12.5px] text-foreground uppercase leading-none tracking-[0.02em]">Sarkaz Crossbowman</h3>
        <div className="flex flex-col gap-1">
            <StatBar label="HP" value={6000} max={12000} color="var(--enemy-elite)" />
            <StatBar label="ATK" value={450} max={1200} color={ATK_COLOR} />
            <StatBar label="DEF" value={200} max={700} color={DEF_COLOR} />
        </div>
    </div>
);

export const ThreatTierColors = () => (
    <div className="grid w-full max-w-2xl grid-cols-3 gap-4">
        <div className="rounded-md border border-border bg-card px-3 py-2.5">
            <div className="mb-2 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.14em]">Normal · B1</div>
            <div className="flex flex-col gap-1">
                <StatBar label="HP" value={550} max={4500} color="var(--muted-foreground)" />
                <StatBar label="ATK" value={130} max={800} color={ATK_COLOR} />
                <StatBar label="DEF" value={0} max={400} color={DEF_COLOR} />
            </div>
        </div>
        <div className="rounded-md border border-border bg-card px-3 py-2.5">
            <div className="mb-2 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.14em]">Elite · S3</div>
            <div className="flex flex-col gap-1">
                <StatBar label="HP" value={6000} max={12000} color="var(--enemy-elite)" />
                <StatBar label="ATK" value={450} max={1200} color={ATK_COLOR} />
                <StatBar label="DEF" value={200} max={700} color={DEF_COLOR} />
            </div>
        </div>
        <div className="rounded-md border border-border bg-card px-3 py-2.5">
            <div className="mb-2 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.14em]">Boss · MD</div>
            <div className="flex flex-col gap-1">
                <StatBar label="HP" value={50000} max={60000} color="var(--primary)" />
                <StatBar label="ATK" value={640} max={3000} color={ATK_COLOR} />
                <StatBar label="DEF" value={520} max={1000} color={DEF_COLOR} />
            </div>
        </div>
    </div>
);

export const ValueFormatting = () => (
    <div className="flex w-72 flex-col gap-2">
        <StatBar label="HP" value={550} max={80000} color="var(--primary)" />
        <StatBar label="HP" value={6000} max={80000} color="var(--primary)" />
        <StatBar label="HP" value={28000} max={80000} color="var(--primary)" />
        <StatBar label="HP" value={80000} max={80000} color="var(--primary)" />
    </div>
);
