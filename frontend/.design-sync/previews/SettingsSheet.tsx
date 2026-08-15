import { SettingsSheet } from "frontend";
import type { ReactNode } from "react";

// The randomizer's full settings surface: a right-hand inset sheet with three
// tabs (Operators / Stages / Roster). It takes `open` as a prop, so the stories
// below render it settled open. `.ds-single` makes the story root the containing
// block for the fixed sheet, hence the explicit stage height.

type Op = {
    id: string;
    name: string;
    rarity: number;
    profession: string;
    subProfessionId: string;
    position: string;
    race: string;
    hasOffensiveRecovery: boolean;
    hasDefensiveRecovery: boolean;
    allSkillsManual: boolean;
};

const op = (id: string, name: string, rarity: number, profession: string, subProfessionId: string, position: string, race: string): Op => ({
    id,
    name,
    rarity,
    profession,
    subProfessionId,
    position,
    race,
    hasOffensiveRecovery: false,
    hasDefensiveRecovery: false,
    allSkillsManual: false,
});

const OPERATORS: Op[] = [
    op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "MELEE", "Kuranta"),
    op("char_263_skadi", "Skadi", 6, "WARRIOR", "fearless", "MELEE", "Unknown"),
    op("char_180_amgoat", "Eyjafjalla", 6, "CASTER", "corecaster", "RANGED", "Caprinae"),
    op("char_249_mlyss", "Muelsyse", 6, "PIONEER", "tactician", "RANGED", "Elf"),
    op("char_103_angel", "Exusiai", 6, "SNIPER", "fastshot", "RANGED", "Sankta"),
    op("char_202_demkni", "Saria", 6, "TANK", "guardian", "MELEE", "Vouivre"),
    op("char_102_texas", "Texas", 5, "PIONEER", "pioneer", "MELEE", "Lupo"),
    op("char_140_whitew", "Lappland", 5, "WARRIOR", "lord", "MELEE", "Lupo"),
    op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "ringhealer", "RANGED", "Liberi"),
    op("char_143_ghost", "Specter", 5, "WARRIOR", "centurion", "MELEE", "Undisclosed"),
    op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "MELEE", "Durin"),
    op("char_150_snakek", "Cuora", 4, "TANK", "protector", "MELEE", "Petram"),
    op("char_124_kroos", "Kroos", 3, "SNIPER", "fastshot", "RANGED", "Cautus"),
    op("char_502_nblade", "Yato", 2, "PIONEER", "pioneer", "MELEE", "Oni"),
];

const STAGES = [
    ["main_08-07", "main_8", "R8-7", "Baptism by Tractor Fire", 18],
    ["main_08-06", "main_8", "R8-6", "War, Sprawling Without Cease", 18],
    ["main_11-14", "main_11", "11-16", "Glory's Hunting Grounds", 21],
    ["main_11-13", "main_11", "11-15", "The Earth Shakes", 24],
    ["act29side_09", "act29sre_zone1", "ZT-9", "Fantasia 'The Gift'", 21],
    ["act29side_ex07", "act29sre_zone2", "ZT-EX-7", "Atonality", 20],
].map(([stageId, zoneId, code, name, apCost]) => ({
    stageId: stageId as string,
    zoneId: zoneId as string,
    code: code as string,
    name: name as string,
    stageType: "MAIN",
    difficulty: "NORMAL",
    apCost: apCost as number,
    canPractice: true,
    canBattleReplay: true,
    canMultipleBattle: true,
    isStoryOnly: false,
    isPredefined: false,
    dangerLevel: "Elite 2 Lv. 30",
    dangerPoint: -1,
    expGain: 200,
    goldGain: 200,
    unlockCondition: [],
    bossMark: false,
}));

const ZONES = [
    { zoneId: "main_8", zoneIndex: 4, type: "MAINLINE", zoneNameFirst: "Episode 8", zoneNameSecond: "Roaring Flare", zoneNameTitleCurrent: "08", canPreview: true, hasAdditionalPanel: false },
    { zoneId: "main_11", zoneIndex: 2, type: "MAINLINE", zoneNameFirst: "Episode 11", zoneNameSecond: "Return To Mist", zoneNameTitleCurrent: "11", canPreview: true, hasAdditionalPanel: true },
    { zoneId: "act29sre_zone1", zoneIndex: 0, type: "ACTIVITY", zoneNameSecond: "Zwillingstürme anbeten", canPreview: false, hasAdditionalPanel: false },
    { zoneId: "act29sre_zone2", zoneIndex: 1, type: "ACTIVITY", zoneNameSecond: "Die Gnade ist ewig", canPreview: false, hasAdditionalPanel: false },
];

const ACTIVITY_LOOKUP = {
    activityById: new Map([["act29side", { id: "act29side", name: "Zwillingstürme im Herbst", startTime: 1714496400, endTime: 1716289199, hasStage: true, isReplicate: false }]]),
    retroByZonePrefix: new Map(),
    retroByActivityId: new Map(),
};

const settings = (over: Record<string, unknown> = {}) => ({
    allowedClasses: ["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER", "MEDIC", "SUPPORT", "SPECIAL"],
    allowedRarities: [6, 5, 4, 3, 2, 1],
    allowedZoneTypes: ["MAINLINE", "ACTIVITY"],
    squadSize: 12,
    allowDuplicates: false,
    hideUnplayableOperators: true,
    onlyOwnedOperators: false,
    onlyCompletedStages: false,
    onlyAvailableStages: true,
    onlyE2Operators: false,
    deselectedStageIds: [],
    ...over,
});

const noop = () => {};
const ids = new Set(OPERATORS.map((o) => o.id));

const Stage = ({ children }: { children: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

const common = {
    onOpenChange: noop,
    onChange: noop,
    allOperators: OPERATORS,
    rosterPickerOperators: OPERATORS,
    rosterSelection: ids,
    rosterIsExplicit: false,
    onRosterChange: noop,
    onRosterReset: noop,
    stages: STAGES,
    zones: ZONES,
    activityLookup: ACTIVITY_LOOKUP,
    stageClears: null,
};

export const OpenOnOperators = () => (
    <Stage>
        <SettingsSheet {...common} hasProfile open rosterIndex={{ owned: ids, e2: new Set([...ids].slice(0, 6)) }} settings={settings()} />
    </Stage>
);

// Signed out: the two roster-backed rules render locked and forced off.
export const SignedOut = () => (
    <Stage>
        <SettingsSheet {...common} hasProfile={false} open rosterIndex={{ owned: new Set<string>(), e2: new Set<string>() }} settings={settings()} />
    </Stage>
);

// A constrained draw: four classes, 6★/5★ only, squad of six, duplicates allowed.
export const NarrowedDraw = () => (
    <Stage>
        <SettingsSheet
            {...common}
            hasProfile
            open
            rosterIndex={{ owned: ids, e2: new Set([...ids].slice(0, 6)) }}
            settings={settings({ allowedClasses: ["PIONEER", "WARRIOR", "SNIPER", "MEDIC"], allowedRarities: [6, 5], squadSize: 6, allowDuplicates: true })}
        />
    </Stage>
);
