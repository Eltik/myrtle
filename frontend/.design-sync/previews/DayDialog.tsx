import { DayDialog } from "frontend";
import type { ReactNode } from "react";

// The popup portals into a `fixed inset-0` viewport, so each open story needs a
// stage tall enough for the backdrop. Opening/closing is interaction-only -
// these are the resting open states the calendar shows after a day is clicked.
const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

const op = (id: string, name: string, rarity: number, profession: string, nationId: string) => ({ id, name, rarity: `TIER_${rarity}`, profession, nationId });
const bd = (operator: ReturnType<typeof op>, day: number) => ({ operator, known: true, raw: `May ${day}`, month: 5, day });

const MAY_14 = [
    bd(op("char_291_aglina", "Angelina", 6, "SUPPORT", "siracusa"), 14),
    bd(op("char_4082_qiubai", "Qiubai", 6, "WARRIOR", "yan"), 14),
    bd(op("char_1029_yato2", "Kirin R Yato", 6, "SPECIAL", "rhodes"), 14),
    bd(op("char_154_morgan", "Morgan", 5, "WARRIOR", "victoria"), 14),
    bd(op("char_502_nblade", "Yato", 2, "PIONEER", "rhodes"), 14),
];

const MAY_15 = [bd(op("char_486_takila", "Tequila", 5, "WARRIOR", "bolivar"), 15)];

const TODAY = new Date(2024, 4, 15);
const noop = () => {};

export const DayRoster = () => (
    <Stage>
        <DayDialog open onOpenChange={noop} date={{ year: 2024, month: 5, day: 14 }} ops={MAY_14} today={TODAY} />
    </Stage>
);

export const Today = () => (
    <Stage>
        <DayDialog open onOpenChange={noop} date={{ year: 2024, month: 5, day: 15 }} ops={MAY_15} today={TODAY} />
    </Stage>
);

export const NoBirthdays = () => (
    <Stage>
        <DayDialog open onOpenChange={noop} date={{ year: 2024, month: 5, day: 12 }} ops={[]} today={TODAY} />
    </Stage>
);
