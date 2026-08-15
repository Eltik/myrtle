import { Stars } from "frontend";

// Rarity tiers as the birthday tool reads them: 6★ is the coral-gold top tier,
// 1★ the grey recruit tier. `rarity` is the only prop.
const TIERS = [
    { rarity: 6, label: "Mlynar", note: "Kazimierz · Guard" },
    { rarity: 5, label: "Tequila", note: "Bolivar · Guard" },
    { rarity: 4, label: "Gavial", note: "Rhodes Island · Medic" },
    { rarity: 3, label: "Adnachiel", note: "Rhodes Island · Sniper" },
    { rarity: 2, label: "Yato", note: "Rhodes Island · Vanguard" },
    { rarity: 1, label: "Justice Knight", note: "Rhodes Island · Sniper" },
];

export const RarityLadder = () => (
    <div className="flex max-w-sm flex-col gap-2.5">
        {TIERS.map((t) => (
            <div key={t.rarity} className="flex items-center justify-between gap-4 border-border border-b pb-2.5 last:border-b-0">
                <div className="min-w-0">
                    <div className="font-sans font-semibold text-[14px] text-foreground">{t.label}</div>
                    <div className="font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.06em]">{t.note}</div>
                </div>
                <Stars rarity={t.rarity} />
            </div>
        ))}
    </div>
);

export const SixStar = () => (
    <div className="flex items-center gap-3">
        <span className="font-sans font-semibold text-[15px] text-foreground">Saileach</span>
        <Stars rarity={6} />
    </div>
);

export const BirthdayRoster = () => (
    <div className="flex max-w-md flex-col">
        <div className="mb-2.5 font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.08em]">May 14 · 4 operators</div>
        {[
            { name: "Angelina", rarity: 6 },
            { name: "Qiubai", rarity: 6 },
            { name: "Morgan", rarity: 5 },
            { name: "Yato", rarity: 2 },
        ].map((o) => (
            <div key={o.name} className="flex items-center justify-between gap-4 rounded-[10px] px-3 py-2">
                <span className="font-medium font-sans text-[13.5px] text-foreground">{o.name}</span>
                <Stars rarity={o.rarity} />
            </div>
        ))}
    </div>
);
