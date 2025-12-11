// Operator helper functions

import type { Range } from "~/types/api";
import { GridCell } from "~/types/frontend/operators";

// Re-export GridCell for convenience
export { GridCell } from "~/types/frontend/operators";

export const rarityToNumber = (rarity: string): number => {
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

export const formatNationId = (nationId: string): string => {
    const mapping: Record<string, string> = {
        rhodes: "Rhodes Island",
        kazimierz: "Kazimierz",
        columbia: "Columbia",
        laterano: "Laterano",
        victoria: "Victoria",
        sami: "Sami",
        bolivar: "Bolivar",
        iberia: "Iberia",
        siracusa: "Siracusa",
        higashi: "Higashi",
        sargon: "Sargon",
        kjerag: "Kjerag",
        minos: "Minos",
        yan: "Yan",
        lungmen: "Lungmen",
        ursus: "Ursus",
        egir: "Aegir",
        leithanien: "Leithanien",
        rim: "Rim Billiton",
    };
    return mapping[nationId.toLowerCase()] ?? capitalize(nationId);
};

export const formatGroupId = (groupId: string): string => {
    const mapping: Record<string, string> = {
        pinus: "Pinus Sylvestris",
        blacksteel: "Blacksteel",
        karlan: "Karlan Trade",
        sweep: "S.W.E.E.P.",
        rhine: "Rhine Lab",
        penguin: "Penguin Logistics",
        siesta: "Siesta",
        lgd: "L.G.D.",
        glasgow: "Glasgow Gang",
        abyssal: "Abyssal Hunters",
        dublinn: "Dublinn",
        elite: "Elite Operators",
        sui: "Yan Sui",
    };
    return mapping[groupId.toLowerCase()] ?? groupId;
};

export const formatProfession = (profession: string): string => {
    const mapping: Record<string, string> = {
        pioneer: "Vanguard",
        tank: "Defender",
        sniper: "Sniper",
        warrior: "Guard",
        caster: "Caster",
        support: "Supporter",
        special: "Specialist",
        medic: "Medic",
    };
    return mapping[profession.toLowerCase()] ?? profession;
};

export const formatSubProfession = (subProfession: string): string => {
    const mapping: Record<string, string> = {
        physician: "ST Medic",
        fearless: "Dreadnought",
        executor: "Executor",
        fastshot: "Marksman Sniper",
        bombarder: "Flinger",
        bard: "Bard",
        protector: "Protector",
        ritualist: "Ritualist",
        pioneer: "Pioneer",
        corecaster: "Core Caster",
        splashcaster: "AOE Caster",
        charger: "Charger",
        centurion: "Centurion",
        guardian: "Guardian",
        slower: "Decel Binder",
        funnel: "Mech-Accord Caster",
        mystic: "Mystic Caster",
        chain: "Chain Caster",
        aoesniper: "AOE Sniper",
        reaperrange: "Spreadshooter",
        longrange: "Deadeye Sniper",
        closerange: "Heavyshooter",
        siegesniper: "Besieger",
        loopshooter: "Loopshooter",
        bearer: "Flag Bearer",
        tactician: "Tactician",
        instructor: "Instructor",
        lord: "Lord",
        artsfghter: "Arts Fighter",
        sword: "Swordmaster",
        musha: "Musha",
        crusher: "Crusher",
        reaper: "Reaper",
        merchant: "Merchant",
        hookmaster: "Hookmaster",
        ringhealer: "AOE Medic",
        healer: "Therapist",
        wandermedic: "Wandering Medic",
        unyield: "Juggernaut",
        artsprotector: "Arts Protector",
        summoner: "Summoner",
        craftsman: "Artificer",
        stalker: "Ambusher",
        pusher: "Pusher",
        dollkeeper: "Dollkeeper",
        agent: "Agent",
        fighter: "Brawler",
        librator: "Liberator",
        hammer: "Earthshaker",
        phalanx: "Phalanx",
        blastcaster: "Blast Caster",
        primcaster: "Primal Caster",
        incantationmedic: "Incantation Medic",
        chainhealer: "Chain Healer",
        shotprotector: "Sentinel",
        fortress: "Fortress",
        duelist: "Duelist",
        hunter: "Hunter",
        geek: "Geek",
        underminer: "Hexer",
        blessing: "Abjurer",
        traper: "Trapmaster",
        alchemist: "Alchemist",
    };
    return mapping[subProfession.toLowerCase()] ?? subProfession;
};

export function capitalize(s: string): string {
    if (!s) return "";
    s = s.toLowerCase();
    return (s[0]?.toUpperCase() ?? "") + s.slice(1);
}

export function getAvatarById(charId: string): string {
    const normalized = normalizeSkinId(charId);
    return `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${normalized}.png`;
}

export function normalizeSkinId(skinId: string): string {
    if (skinId.includes("@")) {
        return encodeURIComponent(skinId.replaceAll("@", "_"));
    } else {
        return encodeURIComponent(skinId.replaceAll("#", "_"));
    }
}

export function getOperatorImage(charId: string, phase: number): string {
    const suffix = phase > 1 ? "2b" : "1b";
    return `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/skin/${charId}_${suffix}.png`;
}

export function getProfessionImage(profession: string): string {
    const formatted = formatProfession(profession).toLowerCase();
    return `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/classes/class_${formatted === "supporter" ? "support" : formatted}.png`;
}

export function getSubProfessionImage(subProfession: string): string {
    return `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/subclass/sub_${subProfession}_icon.png`;
}

export function getEliteImage(elite: number): string {
    return `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${elite}.png`;
}

/**
 * Insert blackboard values into description text
 */
export function insertBlackboard(text: string, blackboard: Array<{ key: string; value: number; valueStr?: string | null }>): string | null {
    const formatVariable = (chunk: string, bb: typeof blackboard) => {
        const tag = chunk.split(":")[0]?.toLowerCase() ?? "";
        const negative = tag.startsWith("-");
        const key = negative ? tag.slice(1) : tag;
        const variable = bb.find((v) => v.key === key);

        if (!variable) return chunk;

        const value = negative ? -variable.value : variable.value;
        return chunk.endsWith("%") ? `<b>${Math.round(value * 100)}%</b>` : `${value}`;
    };

    const textArr = removeStyleTags(text.trim()).split(/{|}/);

    if (textArr.join("") === "") return null;

    for (let i = 0; i < textArr.length; i++) {
        textArr[i] = formatVariable(textArr[i] ?? "", blackboard);
    }

    return textArr.join("").replaceAll("-<b>", "<b>-").replaceAll("+<b>", "<b>+");
}

export function removeStyleTags(text: string): string {
    return text.replace(/<.[a-z]{2,5}?\.[^<]+>|<\/[^<]*>|<color=[^>]+>/g, "");
}

/**
 * Normalize range data for grid display
 */
export function normalizeRange(rangeObject: Range): {
    rows: number;
    cols: number;
    grid: GridCell[][];
} {
    const rangeGrids = [...rangeObject.grids, { row: 0, col: 0 }];

    const rowIndices = rangeGrids.map((cell) => cell.row);
    const colIndices = rangeGrids.map((cell) => cell.col);
    const minRowIndex = Math.min(...rowIndices);
    const maxRowIndex = Math.max(...rowIndices);
    const minColIndex = Math.min(...colIndices);
    const maxColIndex = Math.max(...colIndices);

    const rows = maxRowIndex - minRowIndex + 1;
    const cols = maxColIndex - minColIndex + 1;
    const grid: GridCell[][] = Array(rows)
        .fill(GridCell.empty)
        .map(() => Array(cols).fill(GridCell.empty) as GridCell[]);

    rangeGrids.forEach((cell) => {
        const type = cell.row === 0 && cell.col === 0 ? GridCell.Operator : GridCell.active;
        const rowIdx = cell.row - minRowIndex;
        const colIdx = cell.col - minColIndex;
        if (grid[rowIdx]) {
            grid[rowIdx]![colIdx] = type;
        }
    });

    return { rows, cols, grid };
}
