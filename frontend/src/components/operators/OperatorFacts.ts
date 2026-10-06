import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./OperatorFacts.messages";

type FactT = TypedT<typeof messages>;

// The label functions below spell every `t()` call out as a literal on purpose:
// the extractor only sees literal keys, so a computed `facts.race.${value}`
// would never reach the catalog.

function raceLabel(t: FactT, value: string): string {
    switch (value) {
        case "Undisclosed":
            return t("facts.race.Undisclosed");
        case "Zalak":
            return t("facts.race.Zalak");
        case "Oni":
            return t("facts.race.Oni");
        case "Savra":
            return t("facts.race.Savra");
        case "Durin":
            return t("facts.race.Durin");
        case "Kuranta":
            return t("facts.race.Kuranta");
        case "Vouivre":
            return t("facts.race.Vouivre");
        case "Liberi":
            return t("facts.race.Liberi");
        case "Feline":
            return t("facts.race.Feline");
        case "Cautus":
            return t("facts.race.Cautus");
        case "Perro":
            return t("facts.race.Perro");
        case "Reproba":
            return t("facts.race.Reproba");
        case "Sankta":
            return t("facts.race.Sankta");
        case "Sarkaz":
            return t("facts.race.Sarkaz");
        case "Vulpo":
            return t("facts.race.Vulpo");
        case "Elafia":
            return t("facts.race.Elafia");
        case "Phidia":
            return t("facts.race.Phidia");
        case "Ægir":
            return t("facts.race.Aegir");
        case "Anaty":
            return t("facts.race.Anaty");
        case "Itra":
            return t("facts.race.Itra");
        case "Unknown (Suspected Liberi)":
            return t("facts.race.UnknownSuspectedLiberi");
        case "Archosauria":
            return t("facts.race.Archosauria");
        case "Unknown":
            return t("facts.race.Unknown");
        case "Lupo":
            return t("facts.race.Lupo");
        case "Forte":
            return t("facts.race.Forte");
        case "Ursus":
            return t("facts.race.Ursus");
        case "Petram":
            return t("facts.race.Petram");
        case "Cerato":
            return t("facts.race.Cerato");
        case "Caprinae":
            return t("facts.race.Caprinae");
        case "Draco":
            return t("facts.race.Draco");
        case "Anura":
            return t("facts.race.Anura");
        case "Anasa":
            return t("facts.race.Anasa");
        case "Cautus/Chimera":
            return t("facts.race.CautusChimera");
        case "Kylin":
            return t("facts.race.Kylin");
        case "Pilosa":
            return t("facts.race.Pilosa");
        case "Unknown as requested by management agency":
            return t("facts.race.Unknownasrequestedbymanagementagency");
        case "Manticore":
            return t("facts.race.Manticore");
        case "Lung":
            return t("facts.race.Lung");
        case "Aslan":
            return t("facts.race.Aslan");
        case "Elf":
            return t("facts.race.Elf");
        default:
            return value;
    }
}

function genderLabel(t: FactT, value: string): string {
    switch (value) {
        case "Unknown":
            return t("facts.gender.Unknown");
        case "Female":
            return t("facts.gender.Female");
        case "Male":
            return t("facts.gender.Male");
        case "Male]":
            return t("facts.gender.MaleBugged");
        case "Conviction":
            return t("facts.gender.Conviction");
        default:
            return value;
    }
}

function birthPlaceLabel(t: FactT, value: string): string {
    switch (value) {
        case "Unknown":
            return t("facts.birthPlace.Unknown");
        case "Undisclosed":
            return t("facts.birthPlace.Undisclosed");
        case "Higashi":
            return t("facts.birthPlace.Higashi");
        case "Kazimierz":
            return t("facts.birthPlace.Kazimierz");
        case "Vouivre":
            return t("facts.birthPlace.Vouivre");
        case "Laterano":
            return t("facts.birthPlace.Laterano");
        case "Victoria":
            return t("facts.birthPlace.Victoria");
        case "Rim Billiton":
            return t("facts.birthPlace.RimBilliton");
        case "Leithanien":
            return t("facts.birthPlace.Leithanien");
        case "Bolívar":
            return t("facts.birthPlace.Bolvar");
        case "Sargon":
            return t("facts.birthPlace.Sargon");
        case "Kjerag":
            return t("facts.birthPlace.Kjerag");
        case "Columbia":
            return t("facts.birthPlace.Columbia");
        case "Sami":
            return t("facts.birthPlace.Sami");
        case "Iberia":
            return t("facts.birthPlace.Iberia");
        case "Kazdel":
            return t("facts.birthPlace.Kazdel");
        case "Minos":
            return t("facts.birthPlace.Minos");
        case "Lungmen":
            return t("facts.birthPlace.Lungmen");
        case "Siracusa":
            return t("facts.birthPlace.Siracusa");
        case "Yan":
            return t("facts.birthPlace.Yan");
        case "Ursus":
            return t("facts.birthPlace.Ursus");
        case "Siesta":
            return t("facts.birthPlace.Siesta");
        case "RIM Billiton":
            return t("facts.birthPlace.RIMBilliton");
        case "Ægir":
            return t("facts.birthPlace.Aegir");
        case "Durin":
            return t("facts.birthPlace.Durin");
        case "Siesta (Independent City)":
            return t("facts.birthPlace.SiestaIndependentCity");
        case "Ægir Region":
            return t("facts.birthPlace.AegirRegion");
        case "Unknown as requested by management agency":
            return t("facts.birthPlace.Unknownasrequestedbymanagementagency");
        case "Rhodes Island":
            return t("facts.birthPlace.RhodesIsland");
        case "Far East":
            return t("facts.birthPlace.FarEast");
        default:
            return value;
    }
}

export interface IOperatorFactLabels {
    race: (value: string) => string;
    gender: (value: string) => string;
    birthPlace: (value: string) => string;
}

/**
 * Label functions for the enum-like facts the backend sends as raw English values
 * (race, gender, place of birth). A value outside the known lists, which a KR or JP
 * client can produce, is returned as it came.
 */
export function useOperatorFactLabel(): IOperatorFactLabels {
    const t: FactT = useT("operators");
    return {
        race: (value) => raceLabel(t, value),
        gender: (value) => genderLabel(t, value),
        birthPlace: (value) => birthPlaceLabel(t, value),
    };
}
