import { OperatorRow } from "frontend";

const op = (id: string, name: string, rarity: number, profession: string, nationId: string) => ({ id, name, rarity: `TIER_${rarity}`, profession, nationId });
const bd = (operator: ReturnType<typeof op>, month: number, day: number, raw: string) => ({ operator, known: true, raw, month, day });

const ANGELINA = bd(op("char_291_aglina", "Angelina", 6, "SUPPORT", "siracusa"), 5, 14, "May 14");
const QIUBAI = bd(op("char_4082_qiubai", "Qiubai", 6, "WARRIOR", "yan"), 5, 14, "May 14");
const MORGAN = bd(op("char_154_morgan", "Morgan", 5, "WARRIOR", "victoria"), 5, 14, "May 14");
const YATO = bd(op("char_502_nblade", "Yato", 2, "PIONEER", "rhodes"), 5, 14, "May 14");
const SILENCE2 = bd(op("char_1031_slent2", "Silence the Paradigmatic", 6, "SUPPORT", "columbia"), 5, 18, "May 18");
const GVIAL2 = bd(op("char_1026_gvial2", "Gavial the Invincible", 6, "WARRIOR", "rhodes"), 5, 1, "May 1");
const NIGHTINGALE = bd(op("char_179_cgbird", "Nightingale", 6, "MEDIC", "rhodes"), 5, 4, "May 4");
const TEQUILA = bd(op("char_486_takila", "Tequila", 5, "WARRIOR", "bolivar"), 5, 15, "May 15");

export const SingleOperator = () => (
    <div className="max-w-md">
        <OperatorRow birthday={ANGELINA} />
    </div>
);

export const AlterWithAppellation = () => (
    <div className="flex max-w-md flex-col gap-0.5">
        <OperatorRow birthday={SILENCE2} />
        <OperatorRow birthday={GVIAL2} />
    </div>
);

export const DayRoster = () => (
    <div className="flex max-w-md flex-col gap-0.5">
        <div className="mb-1 px-3 font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">Tuesday · 4 operators</div>
        <OperatorRow birthday={ANGELINA} />
        <OperatorRow birthday={QIUBAI} />
        <OperatorRow birthday={MORGAN} />
        <OperatorRow birthday={YATO} />
    </div>
);

export const RarityMix = () => (
    <div className="flex max-w-md flex-col gap-0.5">
        <OperatorRow birthday={NIGHTINGALE} />
        <OperatorRow birthday={TEQUILA} />
        <OperatorRow birthday={YATO} />
    </div>
);
