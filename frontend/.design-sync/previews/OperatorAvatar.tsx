import { OperatorAvatar } from "frontend";

const SQUAD = [
    { id: "char_4064_mlynar", name: "Mlynar", rarity: 6 },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", rarity: 6 },
    { id: "char_180_amgoat", name: "Eyjafjalla", rarity: 6 },
    { id: "char_1028_texas2", name: "Texas the Omertosa", rarity: 6 },
    { id: "char_102_texas", name: "Texas", rarity: 5 },
];

export const RosterChips = () => (
    <div className="flex flex-wrap gap-2.5">
        {SQUAD.map((op) => (
            <span
                className="relative inline-flex size-12 shrink-0 items-center justify-center overflow-hidden rounded-xl font-bold text-[17px] text-white leading-none tracking-tight"
                key={op.id}
                style={{ backgroundColor: `var(--rarity-${op.rarity})` }}
                title={op.name}
            >
                <OperatorAvatar charId={op.id} name={op.name} />
            </span>
        ))}
    </div>
);

export const Sizes = () => (
    <div className="flex items-end gap-4">
        {[
            { cls: "size-6 rounded-md text-[11px]", label: "sm" },
            { cls: "size-9 rounded-lg text-[14px]", label: "default" },
            { cls: "size-12 rounded-xl text-[17px]", label: "lg" },
            { cls: "size-16 rounded-2xl text-[22px]", label: "xl" },
        ].map((size) => (
            <div className="flex flex-col items-center gap-2" key={size.label}>
                <span className={`relative inline-flex shrink-0 items-center justify-center overflow-hidden font-bold text-white leading-none ${size.cls}`} style={{ backgroundColor: "var(--rarity-6)" }}>
                    <OperatorAvatar charId="char_4064_mlynar" name="Mlynar" />
                </span>
                <span className="font-mono text-[10.5px] text-muted-foreground leading-none">{size.label}</span>
            </div>
        ))}
    </div>
);

export const InitialFallback = () => (
    <div className="w-full max-w-sm">
        <div className="flex gap-2.5">
            <span className="relative inline-flex size-12 shrink-0 items-center justify-center overflow-hidden rounded-xl font-bold text-[17px] text-white leading-none" style={{ backgroundColor: "var(--rarity-6)" }}>
                <OperatorAvatar charId="char_1012_skadi2" name="Skadi the Corrupting Heart" />
            </span>
            <span className="relative inline-flex size-12 shrink-0 items-center justify-center overflow-hidden rounded-xl font-bold text-[17px] text-white leading-none" style={{ backgroundColor: "var(--rarity-4)" }}>
                <OperatorAvatar name="Vermeil" />
            </span>
            <span className="relative inline-flex size-12 shrink-0 items-center justify-center overflow-hidden rounded-xl font-bold text-[17px] text-white leading-none" style={{ backgroundColor: "var(--rarity-4)" }}>
                <OperatorAvatar charId={null} name="Pinecone" />
            </span>
        </div>
        <p className="mt-3 text-muted-foreground text-xs">With no `charId` — or on a 404 — the avatar degrades to the operator's initial.</p>
    </div>
);

export const OperatorRow = () => (
    <div className="w-full max-w-sm rounded-xl border border-border bg-card p-4">
        <div className="flex flex-col gap-3.5">
            {SQUAD.slice(0, 4).map((op, index) => (
                <div className="flex items-center gap-2.5" key={op.id}>
                    <span aria-hidden="true" className="inline-flex size-8 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted font-semibold text-[10px]" style={{ boxShadow: `inset 0 0 0 2px var(--rarity-${op.rarity})` }}>
                        <OperatorAvatar charId={op.id} name={op.name} />
                    </span>
                    <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-1.5">
                            <span className="truncate font-semibold text-[13px] leading-tight">{op.name}</span>
                            <span className="shrink-0 rounded bg-muted px-1 py-0.5 font-mono text-[9.5px] text-muted-foreground">#{index + 1}</span>
                        </div>
                        <div className="truncate text-[11px] text-muted-foreground">E2 90 · S3 M3 · Module X-2</div>
                    </div>
                </div>
            ))}
        </div>
    </div>
);

export const TierRow = () => (
    <div className="w-full max-w-sm">
        <div className="mb-2.5 flex items-center justify-between gap-2">
            <span className="inline-flex items-center rounded-[5px] bg-primary px-2 py-0.75 font-bold text-[10.5px] text-primary-foreground leading-none tracking-tight">S+</span>
            <span className="font-medium font-mono text-[11px] text-muted-foreground leading-none tracking-wide">4 picks</span>
        </div>
        <div className="flex min-w-0 flex-wrap gap-1.5">
            {SQUAD.slice(0, 4).map((op) => (
                <span className="inline-flex items-center gap-1.5 rounded-full border border-border bg-muted py-0.75 pr-2.5 pl-0.75 font-medium text-[11.5px] text-foreground leading-none" key={op.id}>
                    <span className="inline-flex size-5 items-center justify-center overflow-hidden rounded-full font-bold text-[9.5px] text-white leading-none" style={{ backgroundColor: `var(--rarity-${op.rarity})` }}>
                        <OperatorAvatar charId={op.id} name={op.name} />
                    </span>
                    <span>{op.name}</span>
                </span>
            ))}
        </div>
    </div>
);
