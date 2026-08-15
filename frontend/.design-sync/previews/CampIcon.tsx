import { CampIcon } from "frontend";

const FACTIONS = [
    { id: "rhodes", label: "Rhodes Island" },
    { id: "penguin", label: "Penguin Logistics" },
    { id: "rhine", label: "Rhine Lab" },
    { id: "karlan", label: "Karlan Trade" },
    { id: "lgd", label: "L.G.D." },
    { id: "abyssal", label: "Abyssal Hunters" },
    { id: "babel", label: "Babel" },
    { id: "sui", label: "Yan Sui" },
    { id: "elite", label: "Elite Operators" },
    { id: "blacksteel", label: "Blacksteel" },
    { id: "pinus", label: "Pinus Sylvestris" },
    { id: "glasgow", label: "Glasgow Gang" },
];

export const Factions = () => (
    <div className="w-fit rounded-xl border border-border bg-card p-4">
        <div className="mb-3 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">Factions</div>
        <div className="grid grid-cols-6 gap-x-6 gap-y-4">
            {FACTIONS.map((f) => (
                <div key={f.id} className="flex flex-col items-center gap-1.5">
                    <CampIcon groupId={f.id} size={28} />
                    <span className="text-center text-[11px] text-muted-foreground leading-tight">{f.label}</span>
                </div>
            ))}
        </div>
    </div>
);

export const CardWatermark = () => (
    <div className="grid w-full grid-cols-5 gap-3">
        {[
            { id: "char_263_skadi", name: "Skadi", camp: "abyssal", color: "#f7a452" },
            { id: "char_102_texas", name: "Texas", camp: "penguin", color: "#f7e79e" },
            { id: "char_128_plosis", name: "Ptilopsis", camp: "rhine", color: "#f7e79e" },
            { id: "char_136_hsguma", name: "Hoshiguma", camp: "lgd", color: "#f7a452" },
            { id: "char_172_svrash", name: "SilverAsh", camp: "karlan", color: "#f7a452" },
        ].map((op) => (
            <div key={op.id} className="relative flex aspect-2/3 overflow-clip rounded-md border border-muted/50 bg-card">
                <div className="absolute -translate-x-8 -translate-y-4">
                    <CampIcon groupId={op.camp} className="opacity-10" size={360} />
                </div>
                <img alt={`${op.name} portrait`} className="absolute inset-0 h-full w-full rounded-lg object-contain" src={`https://api.myrtle.moe/api/assets/portraits/${op.id}_2.png`} />
                <div className="absolute inset-x-0 bottom-0">
                    <div className="h-12 w-full bg-background/80 backdrop-blur-sm" />
                    <h2 className="absolute bottom-1 left-1 font-bold text-xs uppercase opacity-80">{op.name}</h2>
                    <div className="absolute bottom-0 h-0.5 w-full" style={{ backgroundColor: op.color }} />
                </div>
            </div>
        ))}
    </div>
);

export const FactionFilterOptions = () => (
    <div className="w-72 overflow-hidden rounded-lg border border-border bg-popover p-1 shadow-md">
        <div className="px-2 py-1.5 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">Faction</div>
        {FACTIONS.slice(0, 6).map((f, i) => (
            <div key={f.id} className={i === 1 ? "flex items-center gap-2 rounded-md bg-accent px-2 py-1.5 text-foreground text-sm" : "flex items-center gap-2 rounded-md px-2 py-1.5 text-foreground text-sm"}>
                <CampIcon groupId={f.id} size={18} />
                <span>{f.label}</span>
            </div>
        ))}
    </div>
);

export const Sizes = () => (
    <div className="flex w-fit items-end gap-6 rounded-xl border border-border bg-card p-4">
        {[16, 24, 36, 56].map((size) => (
            <div key={size} className="flex flex-col items-center gap-2">
                <CampIcon groupId="penguin" size={size} />
                <span className="font-mono text-[10px] text-muted-foreground leading-none">{size}px</span>
            </div>
        ))}
    </div>
);
