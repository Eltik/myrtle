import { ClassIcon } from "frontend";

const CLASSES = [
    { profession: "PIONEER", label: "Vanguard" },
    { profession: "WARRIOR", label: "Guard" },
    { profession: "TANK", label: "Defender" },
    { profession: "SNIPER", label: "Sniper" },
    { profession: "CASTER", label: "Caster" },
    { profession: "SUPPORT", label: "Supporter" },
    { profession: "MEDIC", label: "Medic" },
    { profession: "SPECIAL", label: "Specialist" },
];

export const AllClasses = () => (
    <div className="w-fit rounded-xl border border-border bg-card p-4">
        <div className="mb-3 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">Operator classes</div>
        <div className="grid grid-cols-4 gap-x-6 gap-y-4">
            {CLASSES.map((c) => (
                <div key={c.profession} className="flex flex-col items-center gap-1.5">
                    <ClassIcon profession={c.profession} size={28} />
                    <span className="text-[11px] text-muted-foreground leading-none">{c.label}</span>
                </div>
            ))}
        </div>
    </div>
);

export const Sizes = () => (
    <div className="flex w-fit items-end gap-6 rounded-xl border border-border bg-card p-4">
        {[14, 20, 28, 40].map((size) => (
            <div key={size} className="flex flex-col items-center gap-2">
                <ClassIcon profession="SNIPER" size={size} />
                <span className="font-mono text-[10px] text-muted-foreground leading-none">{size}px</span>
            </div>
        ))}
    </div>
);

export const ClassFilterRow = () => (
    <div className="w-fit rounded-xl border border-border bg-card p-4">
        <div className="mb-2 font-medium text-[12px] text-muted-foreground leading-none">Class</div>
        <div className="flex flex-wrap gap-1.5">
            {CLASSES.map((c, i) => (
                <span
                    key={c.profession}
                    title={c.label}
                    className={
                        i === 1 || i === 3
                            ? "inline-flex h-9 w-9 items-center justify-center rounded-md border border-primary bg-primary/15 text-foreground"
                            : "inline-flex h-9 w-9 items-center justify-center rounded-md border border-border bg-background opacity-70"
                    }
                >
                    <ClassIcon profession={c.profession} size={20} />
                </span>
            ))}
        </div>
        <p className="mt-2.5 text-[11px] text-muted-foreground leading-none">2 of 8 classes selected · 96 operators</p>
    </div>
);

export const InResultsRows = () => (
    <div className="flex w-full max-w-2xl flex-col gap-1">
        {[
            { id: "char_4064_mlynar", name: "Młynar", profession: "WARRIOR", label: "Guard", archetype: "Liberator", color: "#f7a452" },
            { id: "char_136_hsguma", name: "Hoshiguma", profession: "TANK", label: "Defender", archetype: "Protector", color: "#f7a452" },
            { id: "char_128_plosis", name: "Ptilopsis", profession: "MEDIC", label: "Medic", archetype: "Multi-target", color: "#f7e79e" },
            { id: "char_151_myrtle", name: "Myrtle", profession: "PIONEER", label: "Vanguard", archetype: "Standard Bearer", color: "#bcabdb" },
        ].map((op) => (
            <div key={op.id} className="relative flex items-center gap-3 rounded-lg border border-transparent bg-card/50 px-3 py-2.5">
                <div className="absolute top-1/2 left-0 h-8 w-0.5 -translate-y-1/2 rounded-full opacity-60" style={{ backgroundColor: op.color }} />
                <img alt={op.name} className="h-12 w-12 rounded-md border border-border/50 bg-background object-cover" src={`https://api.myrtle.moe/api/avatar/${op.id}`} />
                <span className="flex-1 truncate font-semibold text-foreground text-sm uppercase tracking-wide">{op.name}</span>
                <div className="flex w-32 items-center gap-2">
                    <ClassIcon profession={op.profession} size={20} className="opacity-60" />
                    <span className="truncate text-muted-foreground text-sm">{op.label}</span>
                </div>
                <span className="w-32 truncate text-muted-foreground text-sm">{op.archetype}</span>
            </div>
        ))}
    </div>
);
