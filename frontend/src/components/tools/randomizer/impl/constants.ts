import type { OperatorProfession, OperatorRarity } from "#/types/operators";

export const ALL_CLASSES: OperatorProfession[] = ["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER", "MEDIC", "SUPPORT", "SPECIAL"];

export const ALL_RARITIES: OperatorRarity[] = [6, 5, 4, 3, 2, 1];

export const CLASS_LABEL: Record<OperatorProfession, string> = {
    PIONEER: "Vanguard",
    WARRIOR: "Guard",
    TANK: "Defender",
    SNIPER: "Sniper",
    CASTER: "Caster",
    MEDIC: "Medic",
    SUPPORT: "Supporter",
    SPECIAL: "Specialist",
    TOKEN: "Token",
    TRAP: "Trap",
};

/** Operators that are not playable through normal gameplay (event-only, support-only, etc.). */
export const UNPLAYABLE_OPERATOR_IDS = new Set([
    "char_609_acguad",
    "char_608_acpion",
    "char_610_acfend",
    "char_611_acnipe",
    "char_612_accast",
    "char_614_acsupo",
    "char_613_acmedc",
    "char_615_acspec",
    "char_513_apionr",
    "char_508_aguard",
    "char_4025_aprot2",
    "char_511_asnipe",
    "char_509_acast",
    "char_510_amedic",
    "char_600_cpione",
    "char_601_cguard",
    "char_602_cdfend",
    "char_603_csnipe",
    "char_604_ccast",
    "char_606_csuppo",
    "char_605_cmedic",
    "char_607_cspec",
    "char_504_rguard",
    "char_514_rdfend",
    "char_507_rsnipe",
    "char_505_rcast",
    "char_506_rmedic",
]);

export const STORAGE_KEY_SETTINGS = "randomizer-settings-v3";

export const SETTINGS_VERSION = 1;

export const DEFAULT_SETTINGS = {
    allowedClasses: [...ALL_CLASSES] as OperatorProfession[],
    allowedRarities: [...ALL_RARITIES] as OperatorRarity[],
    allowedZoneTypes: ["MAINLINE", "ACTIVITY"],
    squadSize: 12,
    allowDuplicates: false,
    hideUnplayableOperators: true,
    onlyOwnedOperators: false,
    onlyCompletedStages: false,
    onlyAvailableStages: true,
    onlyE2Operators: false,
    deselectedStageIds: [] as string[],
};
