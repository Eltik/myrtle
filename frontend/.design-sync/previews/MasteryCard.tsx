import { MasteryCard } from "frontend";

interface IGapOp {
    id: string;
    operatorId: string;
    name: string;
    charId: string;
    rarity: number;
    sub?: string;
}

const gapOp = (charId: string, name: string, rarity: number, sub: string): IGapOp => ({
    id: `${charId}-gap`,
    operatorId: charId,
    charId,
    name,
    rarity,
    sub,
});

const PENDING_M3 = [gapOp("char_4045_heidi", "Heidi", 5, "E2 60"), gapOp("char_180_amgoat", "Nightingale", 6, "E2 72"), gapOp("char_1023_ghost2", "Specter the Unchained", 6, "E2 80")];

const PENDING_M6 = [gapOp("char_263_skadi", "Skadi", 6, "1 / 3 at M3"), gapOp("char_102_texas", "Texas", 5, "1 / 3 at M3"), gapOp("char_350_surtr", "Surtr", 6, "1 / 3 at M3"), gapOp("char_017_huang", "Blaze", 5, "1 / 3 at M3")];

const PENDING_M9 = [gapOp("char_4064_mlynar", "Mlynar", 6, "2 / 3 at M3"), gapOp("char_293_thorns", "Thorns", 6, "2 / 3 at M3")];

export const Veteran = () => (
    <div className="w-full max-w-md">
        <MasteryCard
            masteries={{
                m3Count: 96,
                m6Count: 61,
                m9Count: 34,
                totalMasteryLevels: 388,
                maxPossibleMasteryLevels: 444,
                e2Count: 148,
                details: { pendingM3: PENDING_M3, pendingM6: PENDING_M6, pendingM9: PENDING_M9 },
            }}
        />
    </div>
);

export const EarlyMasteries = () => (
    <div className="w-full max-w-md">
        <MasteryCard
            masteries={{
                m3Count: 4,
                m6Count: 1,
                m9Count: 0,
                totalMasteryLevels: 17,
                maxPossibleMasteryLevels: 36,
                e2Count: 12,
                details: { pendingM3: PENDING_M3, pendingM6: PENDING_M6, pendingM9: PENDING_M9 },
            }}
        />
    </div>
);

export const AllMastered = () => (
    <div className="w-full max-w-md">
        <MasteryCard
            masteries={{
                m3Count: 148,
                m6Count: 148,
                m9Count: 148,
                totalMasteryLevels: 444,
                maxPossibleMasteryLevels: 444,
                e2Count: 148,
                details: { pendingM3: [], pendingM6: [], pendingM9: [] },
            }}
        />
    </div>
);
