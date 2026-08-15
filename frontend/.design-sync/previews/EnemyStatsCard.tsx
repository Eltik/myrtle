import { EnemyStatsCard } from "frontend";

const ENEMIES = {
    enemy_1034_laxe: {enemyId: "enemy_1034_laxe", name: "Butcher", enemyLevel: "ELITE"},
    enemy_1032_katar: {enemyId: "enemy_1032_katar", name: "Bladed Fighter", enemyLevel: "NORMAL"},
    enemy_1006_shield_2: {enemyId: "enemy_1006_shield_2", name: "Heavy Defender Leader", enemyLevel: "ELITE"},
    enemy_1505_frstar: {enemyId: "enemy_1505_frstar", name: "FrostNova", enemyLevel: "BOSS"},
    enemy_1002_nsabr: {enemyId: "enemy_1002_nsabr", name: "Soldier", enemyLevel: "NORMAL"},
    enemy_1029_shdsbr: {enemyId: "enemy_1029_shdsbr", name: "Shielded Soldier", enemyLevel: "NORMAL"},
    enemy_1011_wizard_2: {enemyId: "enemy_1011_wizard_2", name: "Caster Leader", enemyLevel: "NORMAL"},
};

/** `buildStageEnemyStats` for 4-10: the phase the stage picks, after its ×0.5 move multiplier. */
const STATS = {
    enemy_1505_frstar: {levelIndex: 0, phaseCount: 2, maxHp: 25000, atk: 420, def: 250, res: 50, moveSpeed: 0.25, attackInterval: 3.700000047683716, hasOverride: false},
    enemy_1002_nsabr: {levelIndex: 1, phaseCount: 2, maxHp: 2750, atk: 300, def: 130, res: 0, moveSpeed: 0.550000011920929, attackInterval: 2, hasOverride: false},
    enemy_1032_katar: {levelIndex: 0, phaseCount: 1, maxHp: 4000, atk: 450, def: 300, res: 0, moveSpeed: 0.5, attackInterval: 1.5, hasOverride: false},
    enemy_1029_shdsbr: {levelIndex: 0, phaseCount: 1, maxHp: 2050, atk: 240, def: 250, res: 0, moveSpeed: 0.5, attackInterval: 2, hasOverride: false},
    enemy_1034_laxe: {levelIndex: 0, phaseCount: 1, maxHp: 9000, atk: 850, def: 230, res: 30, moveSpeed: 0.3499999940395355, attackInterval: 3.5, hasOverride: false},
    enemy_1006_shield_2: {levelIndex: 0, phaseCount: 2, maxHp: 10000, atk: 600, def: 1000, res: 0, moveSpeed: 0.375, attackInterval: 2.5999999046325684, hasOverride: false},
    enemy_1011_wizard_2: {levelIndex: 0, phaseCount: 3, maxHp: 2400, atk: 300, def: 80, res: 50, moveSpeed: 0.4000000059604645, attackInterval: 4, hasOverride: false},
};

/** MapView portals this card into a cursor-following popover. */
const Popover = ({ children }: { children?: React.ReactNode }) => <div className="w-fit rounded-md border bg-popover px-3 py-2.5 text-popover-foreground shadow-md">{children}</div>;

export const BossHover = () => (
    <Popover>
        <EnemyStatsCard id="enemy_1505_frstar" enemy={ENEMIES.enemy_1505_frstar} stats={STATS.enemy_1505_frstar} />
    </Popover>
);

export const EliteAndNormal = () => (
    <div className="flex flex-wrap gap-4">
        <Popover>
            <EnemyStatsCard id="enemy_1034_laxe" enemy={ENEMIES.enemy_1034_laxe} stats={STATS.enemy_1034_laxe} />
        </Popover>
        <Popover>
            <EnemyStatsCard id="enemy_1002_nsabr" enemy={ENEMIES.enemy_1002_nsabr} stats={STATS.enemy_1002_nsabr} />
        </Popover>
    </div>
);

/** A multi-phase enemy also prints which stat block the stage selected. */
export const MultiPhase = () => (
    <div className="flex flex-wrap gap-4">
        <Popover>
            <EnemyStatsCard id="enemy_1006_shield_2" enemy={ENEMIES.enemy_1006_shield_2} stats={STATS.enemy_1006_shield_2} />
        </Popover>
        <Popover>
            <EnemyStatsCard id="enemy_1011_wizard_2" enemy={ENEMIES.enemy_1011_wizard_2} stats={STATS.enemy_1011_wizard_2} />
        </Popover>
    </div>
);

/** No handbook entry and no stat block — the card degrades to the raw id. */
export const NoStatData = () => (
    <Popover>
        <EnemyStatsCard id="enemy_1526_sfsui" enemy={null} stats={null} />
    </Popover>
);
