import { TodayCallout } from "frontend";

// The harness clock is fixed to 2024-05-15, which is the date the birthday page
// would render as "today"; the callout returns null when `ops` is empty, so the
// empty case has no card of its own.
const TODAY = new Date(2024, 4, 15);
const MAY_14 = new Date(2024, 4, 14);

const op = (id: string, name: string, rarity: number, profession: string, nationId: string) => ({ id, name, rarity: `TIER_${rarity}`, profession, nationId });
const bd = (operator: ReturnType<typeof op>, month: number, day: number, raw: string) => ({ operator, known: true, raw, month, day });

const TEQUILA = bd(op("char_486_takila", "Tequila", 5, "WARRIOR", "bolivar"), 5, 15, "May 15");
const ANGELINA = bd(op("char_291_aglina", "Angelina", 6, "SUPPORT", "siracusa"), 5, 14, "May 14");
const QIUBAI = bd(op("char_4082_qiubai", "Qiubai", 6, "WARRIOR", "yan"), 5, 14, "May 14");
const KIRIN = bd(op("char_1029_yato2", "Kirin R Yato", 6, "SPECIAL", "rhodes"), 5, 14, "May 14");
const MORGAN = bd(op("char_154_morgan", "Morgan", 5, "WARRIOR", "victoria"), 5, 14, "May 14");
const VENDELA = bd(op("char_494_vendla", "Vendela", 5, "MEDIC", "victoria"), 5, 14, "May 14");
const YATO = bd(op("char_502_nblade", "Yato", 2, "PIONEER", "rhodes"), 5, 14, "May 14");

export const OneBirthday = () => (
    <div className="max-w-2xl">
        <TodayCallout ops={[TEQUILA]} today={TODAY} />
    </div>
);

export const SharedBirthday = () => (
    <div className="max-w-2xl">
        <TodayCallout ops={[ANGELINA, QIUBAI, KIRIN]} today={MAY_14} />
    </div>
);

export const BusyDay = () => (
    <div className="max-w-2xl">
        <TodayCallout ops={[ANGELINA, QIUBAI, KIRIN, MORGAN, VENDELA, YATO]} today={MAY_14} />
    </div>
);
