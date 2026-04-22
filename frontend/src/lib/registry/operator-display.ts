import type { OperatorProfession } from "#/types/operators";

/** Maps the raw profession enum to the CSS class used for the role-colored chip. */
export function professionClass(profession: OperatorProfession): string {
    switch (profession) {
        case "PIONEER":
            return "c-vanguard";
        case "CASTER":
            return "c-caster";
        case "MEDIC":
            return "c-medic";
        case "TANK":
            return "c-defender";
        case "SUPPORT":
            return "c-support";
        case "SNIPER":
            return "c-sniper";
        case "WARRIOR":
            return "c-guard";
        case "SPECIAL":
            return "c-special";
        default:
            return "c-guard";
    }
}

/** Human-readable profession label. */
export function professionLabel(profession: OperatorProfession): string {
    switch (profession) {
        case "PIONEER":
            return "Vanguard";
        case "CASTER":
            return "Caster";
        case "MEDIC":
            return "Medic";
        case "TANK":
            return "Defender";
        case "SUPPORT":
            return "Supporter";
        case "SNIPER":
            return "Sniper";
        case "WARRIOR":
            return "Guard";
        case "SPECIAL":
            return "Specialist";
        case "TOKEN":
            return "Token";
        case "TRAP":
            return "Trap";
        default:
            return profession;
    }
}
