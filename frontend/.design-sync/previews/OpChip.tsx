import { OpChip } from "frontend";

// OpChip only reads `id`, `name` and `rarity` off the operator - the tile is
// tinted with the rarity token and the portrait is fetched from the live API.
const op = (id: string, name: string, rarity: number) => ({ id, name, rarity: `TIER_${rarity}`, profession: "WARRIOR", nationId: "rhodes" });

const ANGELINA = op("char_291_aglina", "Angelina", 6);
const TEQUILA = op("char_486_takila", "Tequila", 5);
const GAVIAL = op("char_187_ccheal", "Gavial", 4);
const MIDNIGHT = op("char_283_midn", "Midnight", 3);
const YATO = op("char_502_nblade", "Yato", 2);
const NIGHTINGALE = op("char_179_cgbird", "Nightingale", 6);
const DOROTHY = op("char_4048_doroth", "Dorothy", 6);
const MORGAN = op("char_154_morgan", "Morgan", 5);
const LISKARM = op("char_107_liskam", "Liskarm", 5);
const CLICK = op("char_328_cammou", "Click", 4);

export const Sizes = () => (
    <div className="flex items-end gap-6">
        {(["sm", "default", "lg", "xl"] as const).map((size) => (
            <div key={size} className="flex flex-col items-center gap-2">
                <OpChip operator={ANGELINA} size={size} />
                <span className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.08em]">{size}</span>
            </div>
        ))}
    </div>
);

export const RarityRange = () => (
    <div className="flex items-center gap-3">
        {[ANGELINA, TEQUILA, GAVIAL, MIDNIGHT, YATO].map((o) => (
            <div key={o.id} className="flex flex-col items-center gap-2">
                <OpChip operator={o} size="lg" />
                <span className="font-medium font-mono text-[10.5px] text-muted-foreground">{o.rarity.replace("TIER_", "")}★</span>
            </div>
        ))}
    </div>
);

export const CalendarCell = () => (
    <div className="flex w-40 flex-col gap-1.5 rounded-[10px] border border-border bg-muted/50 p-2">
        <div className="flex items-center justify-between">
            <span className="font-sans font-semibold text-[14px] text-foreground leading-none">14</span>
            <span className="size-1.25 rounded-full bg-primary" />
        </div>
        <div className="mt-auto flex flex-wrap gap-1">
            {[ANGELINA, NIGHTINGALE, DOROTHY, MORGAN, LISKARM, CLICK, GAVIAL, MIDNIGHT, YATO].map((o) => (
                <OpChip key={o.id} operator={o} size="sm" />
            ))}
            <span className="inline-flex h-5.5 items-center justify-center rounded-md border border-border bg-card px-1.5 font-medium font-mono text-[10.5px] text-muted-foreground">+3</span>
        </div>
    </div>
);
