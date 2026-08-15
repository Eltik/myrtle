import { TeamIcon } from "frontend";

const NATIONS = [
    { id: "rhodes", label: "Rhodes Island" },
    { id: "lungmen", label: "Lungmen" },
    { id: "kazimierz", label: "Kazimierz" },
    { id: "columbia", label: "Columbia" },
    { id: "victoria", label: "Victoria" },
    { id: "leithanien", label: "Leithanien" },
    { id: "siracusa", label: "Siracusa" },
    { id: "kjerag", label: "Kjerag" },
    { id: "yan", label: "Yan" },
    { id: "higashi", label: "Higashi" },
    { id: "ursus", label: "Ursus" },
    { id: "sargon", label: "Sargon" },
];

export const Nations = () => (
    <div className="w-fit rounded-xl border border-border bg-card p-4">
        <div className="mb-3 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">Nations</div>
        <div className="grid grid-cols-6 gap-x-6 gap-y-4">
            {NATIONS.map((n) => (
                <div key={n.id} className="flex flex-col items-center gap-1.5">
                    <TeamIcon teamId={n.id} size={26} />
                    <span className="text-[11px] text-muted-foreground leading-none">{n.label}</span>
                </div>
            ))}
        </div>
    </div>
);

export const NationFilterOptions = () => (
    <div className="w-72 overflow-hidden rounded-lg border border-border bg-popover p-1 shadow-md">
        <div className="px-2 py-1.5 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">Nation</div>
        {NATIONS.slice(0, 6).map((n, i) => (
            <div key={n.id} className={i === 2 ? "flex items-center gap-2 rounded-md bg-accent px-2 py-1.5 text-foreground text-sm" : "flex items-center gap-2 rounded-md px-2 py-1.5 text-foreground text-sm"}>
                <TeamIcon teamId={n.id} size={18} />
                <span>{n.label}</span>
            </div>
        ))}
    </div>
);

export const Sizes = () => (
    <div className="flex w-fit items-end gap-6 rounded-xl border border-border bg-card p-4">
        {[14, 18, 26, 36].map((size) => (
            <div key={size} className="flex flex-col items-center gap-2">
                <TeamIcon teamId="kazimierz" size={size} />
                <span className="font-mono text-[10px] text-muted-foreground leading-none">{size}px</span>
            </div>
        ))}
    </div>
);
