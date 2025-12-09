import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import type { OperatorRarity } from "~/types/api";

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

export const rarityToNumber = (rarity: OperatorRarity): number => {
    switch (rarity) {
        case "TIER_6":
            return 6;
        case "TIER_5":
            return 5;
        case "TIER_4":
            return 4;
        case "TIER_3":
            return 3;
        case "TIER_2":
            return 2;
        case "TIER_1":
            return 1;
        default:
            return 0;
    }
};

export const formatProfession = (profession: string): string => {
    switch (profession.toLowerCase()) {
        case "pioneer":
            return "Vanguard";
        case "tank":
            return "Defender";
        case "sniper":
            return "Sniper";
        case "warrior":
            return "Guard";
        case "caster":
            return "Caster";
        case "support":
            return "Supporter";
        case "special":
            return "Specialist";
        case "medic":
            return "Medic";
        default:
            return profession;
    }
};

export const formatSubProfession = (subProfession: string): string => {
    switch (subProfession) {
        case "physician":
            return "ST Medic";
        case "fearless":
            return "Dreadnought";
        case "executor":
            return "Executor";
        case "fastshot":
            return "Marksman Sniper";
        case "bombarder":
            return "Flinger";
        case "bard":
            return "Bard";
        case "protector":
            return "Protector";
        case "ritualist":
            return "Ritualist";
        case "pioneer":
            return "Pioneer";
        case "corecaster":
            return "Core Caster";
        case "splashcaster":
            return "AOE Caster";
        case "charger":
            return "Charger";
        case "centurion":
            return "Centurion";
        case "guardian":
            return "Guardian";
        case "slower":
            return "Decel Binder";
        case "funnel":
            return "Mech-Accord Caster";
        case "mystic":
            return "Mystic Caster";
        case "chain":
            return "Chain Caster";
        case "aoesniper":
            return "AOE Sniper";
        case "reaperrange":
            return "Spreadshooter";
        case "longrange":
            return "Deadeye Sniper";
        case "closerange":
            return "Heavyshooter";
        case "siegesniper":
            return "Besieger";
        case "loopshooter":
            return "Loopshooter";
        case "bearer":
            return "Flag Bearer";
        case "tactician":
            return "Tactician";
        case "instructor":
            return "Instructor";
        case "lord":
            return "Lord";
        case "artsfghter":
            return "Arts Fighter";
        case "sword":
            return "Swordmaster";
        case "musha":
            return "Musha";
        case "crusher":
            return "Crusher";
        case "reaper":
            return "Reaper";
        case "merchant":
            return "Merchant";
        case "hookmaster":
            return "Hookmaster";
        case "ringhealer":
            return "AOE Medic";
        case "healer":
            return "Therapist";
        case "wandermedic":
            return "Wandering Medic";
        case "unyield":
            return "Juggernaught";
        case "artsprotector":
            return "Arts Protector";
        case "summoner":
            return "Summoner";
        case "craftsman":
            return "Artificer";
        case "stalker":
            return "Ambusher";
        case "pusher":
            return "Pusher";
        case "dollkeeper":
            return "Dollkeeper";
        case "agent":
            return "Agent";
        case "fighter":
            return "Brawler";
        case "librator":
            return "Liberator";
        case "hammer":
            return "Earthshaker";
        case "phalanx":
            return "Phalanx";
        case "blastcaster":
            return "Blast Caster";
        case "primcaster":
            return "Primal Caster";
        case "incantationmedic":
            return "Incantation Medic";
        case "chainhealer":
            return "Chain Healer";
        case "shotprotector":
            return "Sentinel";
        case "fortress":
            return "Fortress";
        case "duelist":
            return "Duelist";
        case "hunter":
            return "Hunter";
        case "geek":
            return "Geek";
        case "underminer":
            return "Hexer";
        case "blessing":
            return "Abjurer";
        case "traper":
            return "Trapmaster";
        case "alchemist":
            return "Alchemist";
        default:
            return subProfession;
    }
};

export const formatNationId = (nationId: string) => {
    switch (nationId.toLowerCase()) {
        case "rhodes":
            return "Rhodes Island";
        case "kazimierz":
            return "Kazimierz";
        case "columbia":
            return "Columbia";
        case "laterano":
            return "Laterano";
        case "victoria":
            return "Victoria";
        case "sami":
            return "Sami";
        case "bolivar":
            return "Bolivar";
        case "iberia":
            return "Iberia";
        case "siracusa":
            return "Siracusa";
        case "higashi":
            return "Higashi";
        case "sargon":
            return "Sargon";
        case "kjerag":
            return "Kjerag";
        case "minos":
            return "Minos";
        case "yan":
            return "Yan";
        case "lungmen":
            return "Lungmen";
        case "ursus":
            return "Ursus";
        case "egir":
            return "Ægir";
        case "leithanien":
            return "Leithanien";
        case "rim":
            return "Rim Billiton";
        default:
            return capitalize(nationId);
    }
};

export const formatGroupId = (groupId: string) => {
    switch (groupId.toLowerCase()) {
        case "pinus":
            return "Pinus Sylvestris";
        case "blacksteel":
            return "Blacksteel";
        case "karlan":
            return "Karlan Trade";
        case "sweep":
            return "S.W.E.E.P.";
        case "rhine":
            return "Rhine Lab";
        case "penguin":
            return "Penguin Logistics";
        case "siesta":
            return "Siesta";
        case "lgd":
            return "L.G.D.";
        case "glasgow":
            return "Glasgow Gang";
        case "abyssal":
            return "Abyssal Hunters";
        case "dublinn":
            return "Dublinn";
        case "elite":
            return "Elite Operators";
        case "sui":
            return "Yan Sui";
        default:
            return groupId;
    }
};

export const formatTeamId = (teamId: string) => {
    switch (teamId.toLowerCase()) {
        case "action4":
            return "Action Team A4";
        case "reserve1":
            return "Reserve Op Team A1";
        case "reserve4":
            return "Reserve Op Team A4";
        case "reserve6":
            return "Reserve Op Team A6";
        case "student":
            return "Ursus Student Self-Governing Group";
        case "chiave":
            return "Chiave's Gang";
        case "rainbow":
            return "Team Rainbow";
        case "followers":
            return "Followers";
        case "lee":
            return "Lee's Detective Agency";
        default:
            return capitalize(teamId);
    }
};

export function capitalize(s: string): string {
    if (!s) return "";
    return s.charAt(0).toUpperCase() + s.slice(1).toLowerCase();
}
