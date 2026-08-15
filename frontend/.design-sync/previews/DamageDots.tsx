import { DamageDots } from "frontend";

// Damage type is encoded as a run of tinted squares (--dmg-physic / --dmg-magic /
// --dmg-heal / --dmg-none) so a dense enemy grid can carry it without text.
const ROWS: { types: ("PHYSIC" | "MAGIC" | "HEAL" | "NO_DAMAGE")[]; label: string; enemy: string }[] = [
    { types: ["PHYSIC"], label: "Physical", enemy: "Originium Slug" },
    { types: ["MAGIC"], label: "Arts", enemy: "Sarkaz Caster" },
    { types: ["PHYSIC", "MAGIC"], label: "Physical · Arts", enemy: "Mandragora" },
    { types: ["HEAL"], label: "Heal", enemy: "Sarkaz Chain Healer" },
    { types: ["NO_DAMAGE"], label: "No Damage", enemy: "Sarkaz Sentinel" },
];

export const DamageTypes = () => (
    <div className="flex flex-col gap-2.5">
        {ROWS.map((r) => (
            <div key={r.label} className="flex items-center gap-3">
                <span className="inline-flex w-10 justify-start">
                    <DamageDots types={r.types} />
                </span>
                <span className="w-32 font-sans text-[12.5px] text-muted-foreground">{r.label}</span>
                <span className="font-sans font-semibold text-[12.5px] text-foreground uppercase tracking-[0.03em]">{r.enemy}</span>
            </div>
        ))}
    </div>
);

export const InlineWithName = () => (
    <div className="flex w-64 flex-col gap-2 rounded-md border border-border bg-card px-3 py-2.5">
        <div className="flex min-h-7 items-start gap-1.5">
            <h3 className="m-0 line-clamp-2 min-w-0 flex-1 font-sans font-semibold text-[12.5px] text-foreground uppercase leading-[1.15] tracking-[0.02em]">Mandragora</h3>
            <DamageDots types={["PHYSIC", "MAGIC"]} />
        </div>
        <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.14em]">MD · Boss</span>
    </div>
);

export const RowScale = () => (
    <div className="flex flex-col gap-4">
        <div className="flex items-center gap-3">
            <span className="w-28 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.14em]">Grid · 6px</span>
            <DamageDots types={["PHYSIC", "MAGIC", "HEAL", "NO_DAMAGE"]} />
        </div>
        <div className="flex items-center gap-3">
            <span className="w-28 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.14em]">List · 7px</span>
            <DamageDots types={["PHYSIC", "MAGIC", "HEAL", "NO_DAMAGE"]} size={7} />
        </div>
        <div className="flex items-center gap-3">
            <span className="w-28 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.14em]">Legend · 12px</span>
            <DamageDots types={["PHYSIC", "MAGIC", "HEAL", "NO_DAMAGE"]} size={12} />
        </div>
    </div>
);
