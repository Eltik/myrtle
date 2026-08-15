import { ExportDialog } from "frontend";

// `enemiesExportSchema` / `operatorsExportSchema` live in #/lib/export and are
// not part of the design bundle, so these previews rebuild the same schema shape
// (id, itemName, pluralName, grouped fields) over real rows.
interface IEnemyRow {
    enemyId: string;
    enemyIndex: string;
    name: string;
    enemyLevel: string;
    damageType: string[];
    race: string | null;
    maxHp: number;
    atk: number;
    def: number;
    res: number;
}

const ENEMY_SCHEMA = {
    id: "enemies",
    itemName: "enemy",
    pluralName: "enemies",
    fields: [
        { id: "enemyId", label: "enemyId", group: "Identity", defaultEnabled: true, accessor: (e: IEnemyRow) => e.enemyId },
        { id: "enemyIndex", label: "enemyIndex", group: "Identity", defaultEnabled: true, accessor: (e: IEnemyRow) => e.enemyIndex },
        { id: "name", label: "name", group: "Identity", defaultEnabled: true, accessor: (e: IEnemyRow) => e.name },
        { id: "enemyLevel", label: "threat", group: "Classification", defaultEnabled: true, accessor: (e: IEnemyRow) => e.enemyLevel },
        { id: "damageType", label: "damageType", group: "Classification", defaultEnabled: true, accessor: (e: IEnemyRow) => e.damageType },
        { id: "race", label: "race", group: "Classification", accessor: (e: IEnemyRow) => e.race },
        { id: "maxHp", label: "hp", group: "Stats (level 0)", defaultEnabled: true, accessor: (e: IEnemyRow) => e.maxHp },
        { id: "atk", label: "atk", group: "Stats (level 0)", defaultEnabled: true, accessor: (e: IEnemyRow) => e.atk },
        { id: "def", label: "def", group: "Stats (level 0)", defaultEnabled: true, accessor: (e: IEnemyRow) => e.def },
        { id: "res", label: "res", group: "Stats (level 0)", defaultEnabled: true, accessor: (e: IEnemyRow) => e.res },
        { id: "imm_stun", label: "stunImmune", group: "Immunities", accessor: () => false },
        { id: "imm_silence", label: "silenceImmune", group: "Immunities", accessor: () => false },
        { id: "imm_frozen", label: "frozenImmune", group: "Immunities", accessor: () => false },
        { id: "portrait", label: "portrait", group: "Media", accessor: (e: IEnemyRow) => `/textures/spritepack/${e.enemyId}.png` },
    ],
};

const ENEMY_ROWS: IEnemyRow[] = [
    { enemyId: "enemy_1007_slime", enemyIndex: "B1", name: "Originium Slug", enemyLevel: "NORMAL", damageType: ["PHYSIC"], race: "Infected Creature", maxHp: 550, atk: 130, def: 0, res: 0 },
    { enemyId: "enemy_1073_dscout", enemyIndex: "S11", name: "Sarkaz Sentinel", enemyLevel: "NORMAL", damageType: ["NO_DAMAGE"], race: "Sarkaz", maxHp: 4000, atk: 0, def: 100, res: 30 },
    { enemyId: "enemy_1012_dcross", enemyIndex: "S3", name: "Sarkaz Crossbowman", enemyLevel: "ELITE", damageType: ["PHYSIC"], race: "Sarkaz", maxHp: 6000, atk: 450, def: 200, res: 50 },
    { enemyId: "enemy_1072_dlancer_2", enemyIndex: "S10", name: "Sarkaz Lancer Leader", enemyLevel: "ELITE", damageType: ["PHYSIC"], race: "Sarkaz", maxHp: 8000, atk: 500, def: 230, res: 40 },
    { enemyId: "enemy_1500_skulsr", enemyIndex: "SS", name: "Skullshatterer", enemyLevel: "BOSS", damageType: ["PHYSIC"], race: null, maxHp: 10500, atk: 1000, def: 150, res: 30 },
    { enemyId: "enemy_1523_mandra", enemyIndex: "MD", name: "Mandragora", enemyLevel: "BOSS", damageType: ["PHYSIC", "MAGIC"], race: null, maxHp: 50000, atk: 640, def: 520, res: 35 },
    { enemyId: "enemy_1525_blkswb", enemyIndex: "BJ", name: "Degenbrecher", enemyLevel: "BOSS", damageType: ["PHYSIC"], race: null, maxHp: 28000, atk: 900, def: 600, res: 50 },
    { enemyId: "enemy_1560_cnvlap", enemyIndex: "LOM", name: "'La Signora del Carnevale'", enemyLevel: "BOSS", damageType: ["PHYSIC", "MAGIC"], race: null, maxHp: 60000, atk: 425, def: 500, res: 30 },
];

// 1,541 catalogued enemies; a Boss + Arts filter narrows it to 48, and the grid
// shows 24 per page.
const ALL_ENEMIES = Array.from({ length: 1541 }, (_, i) => ({ ...ENEMY_ROWS[i % ENEMY_ROWS.length], enemyId: `${ENEMY_ROWS[i % ENEMY_ROWS.length].enemyId}_${i}` }));

interface IOperatorRow {
    id: string;
    name: string;
    rarity: number;
    profession: string;
    subProfessionId: string;
    nationId: string | null;
    hp: number;
    atk: number;
    def: number;
    res: number;
}

const OPERATOR_SCHEMA = {
    id: "operators",
    itemName: "operator",
    pluralName: "operators",
    fields: [
        { id: "id", label: "id", group: "Identity", defaultEnabled: true, accessor: (o: IOperatorRow) => o.id },
        { id: "name", label: "name", group: "Identity", defaultEnabled: true, accessor: (o: IOperatorRow) => o.name },
        { id: "rarity", label: "rarity", group: "Identity", defaultEnabled: true, accessor: (o: IOperatorRow) => o.rarity },
        { id: "profession", label: "profession", group: "Class", defaultEnabled: true, accessor: (o: IOperatorRow) => o.profession },
        { id: "subProfessionId", label: "subProfession", group: "Class", defaultEnabled: true, accessor: (o: IOperatorRow) => o.subProfessionId },
        { id: "nationId", label: "nation", group: "Origin", accessor: (o: IOperatorRow) => o.nationId },
        { id: "hp", label: "hp", group: "Stats (max)", defaultEnabled: true, accessor: (o: IOperatorRow) => o.hp },
        { id: "atk", label: "atk", group: "Stats (max)", defaultEnabled: true, accessor: (o: IOperatorRow) => o.atk },
        { id: "def", label: "def", group: "Stats (max)", defaultEnabled: true, accessor: (o: IOperatorRow) => o.def },
        { id: "res", label: "res", group: "Stats (max)", defaultEnabled: true, accessor: (o: IOperatorRow) => o.res },
    ],
};

const OPERATOR_ROWS: IOperatorRow[] = [
    { id: "char_4064_mlynar", name: "Mlynar", rarity: 6, profession: "WARRIOR", subProfessionId: "sword", nationId: "kazimierz", hp: 3455, atk: 1290, def: 419, res: 0 },
    { id: "char_263_skadi", name: "Skadi", rarity: 6, profession: "WARRIOR", subProfessionId: "centurion", nationId: "rhodes", hp: 3183, atk: 942, def: 411, res: 0 },
    { id: "char_1028_texas2", name: "Texas the Omertosa", rarity: 6, profession: "SPECIAL", subProfessionId: "hookmaster", nationId: "lungmen", hp: 2472, atk: 906, def: 265, res: 0 },
    { id: "char_002_amiya", name: "Amiya", rarity: 5, profession: "CASTER", subProfessionId: "corecaster", nationId: "rhodes", hp: 1447, atk: 730, def: 137, res: 10 },
    { id: "char_102_texas", name: "Texas", rarity: 5, profession: "VANGUARD", subProfessionId: "pioneer", nationId: "lungmen", hp: 1697, atk: 620, def: 258, res: 0 },
    { id: "char_180_amgoat", name: "Eyjafjalla", rarity: 6, profession: "CASTER", subProfessionId: "corecaster", nationId: "rhodes", hp: 1523, atk: 895, def: 133, res: 10 },
];

const ALL_OPERATORS = Array.from({ length: 438 }, (_, i) => ({ ...OPERATOR_ROWS[i % OPERATOR_ROWS.length], id: `${OPERATOR_ROWS[i % OPERATOR_ROWS.length].id}_${i}` }));

const noop = () => undefined;

export const ExportEnemies = () => <ExportDialog open onOpenChange={noop} schema={ENEMY_SCHEMA} allRows={ALL_ENEMIES} filteredRows={ALL_ENEMIES.slice(0, 48)} pageRows={ALL_ENEMIES.slice(0, 24)} title="Enemies" />;

export const ExportOperators = () => <ExportDialog open onOpenChange={noop} schema={OPERATOR_SCHEMA} allRows={ALL_OPERATORS} filteredRows={ALL_OPERATORS.slice(0, 61)} pageRows={ALL_OPERATORS.slice(0, 36)} title="Operators" />;

export const SmallSelection = () => <ExportDialog open onOpenChange={noop} schema={ENEMY_SCHEMA} allRows={ENEMY_ROWS} filteredRows={ENEMY_ROWS.slice(0, 3)} title="Reunion enemies" />;
