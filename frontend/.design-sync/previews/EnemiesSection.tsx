import { EnemiesSection } from "frontend";

/** `GET /stages/main_04-10/detail` → `enemies`, trimmed to what the stage UIs read. */
const ENEMIES = {
    enemy_1034_laxe: {enemyId: "enemy_1034_laxe", name: "Butcher", enemyLevel: "ELITE"},
    enemy_1032_katar: {enemyId: "enemy_1032_katar", name: "Bladed Fighter", enemyLevel: "NORMAL"},
    enemy_1006_shield_2: {enemyId: "enemy_1006_shield_2", name: "Heavy Defender Leader", enemyLevel: "ELITE"},
    enemy_1505_frstar: {enemyId: "enemy_1505_frstar", name: "FrostNova", enemyLevel: "BOSS"},
    enemy_1002_nsabr: {enemyId: "enemy_1002_nsabr", name: "Soldier", enemyLevel: "NORMAL"},
    enemy_1029_shdsbr: {enemyId: "enemy_1029_shdsbr", name: "Shielded Soldier", enemyLevel: "NORMAL"},
    enemy_1011_wizard_2: {enemyId: "enemy_1011_wizard_2", name: "Caster Leader", enemyLevel: "NORMAL"},
};

/** What `tallyEnemies` counts across 4-10's single wave, boss first. */
const TALLY = [
    {id: "enemy_1505_frstar", count: 2},
    {id: "enemy_1034_laxe", count: 4},
    {id: "enemy_1006_shield_2", count: 1},
    {id: "enemy_1002_nsabr", count: 19},
    {id: "enemy_1032_katar", count: 9},
    {id: "enemy_1011_wizard_2", count: 5},
    {id: "enemy_1029_shdsbr", count: 4},
].map((t) => ({ ...t, enemy: ENEMIES[t.id as keyof typeof ENEMIES] ?? null }));

const noop = () => {};

export const StageRoster = () => (
    <div className="max-w-2xl">
        <EnemiesSection tally={TALLY} onFocusEnemy={noop} />
    </div>
);

/** A stage that declares enemies but never spawns them: counted at 0, badge suppressed. */
export const DeclaredOnly = () => (
    <div className="max-w-2xl">
        <EnemiesSection tally={TALLY.slice(0, 4).map((t) => ({ ...t, count: 0 }))} onFocusEnemy={noop} />
    </div>
);

/** An enemy the handbook has no entry for still gets a row, keyed by its raw id. */
export const UnresolvedEnemy = () => (
    <div className="max-w-2xl">
        <EnemiesSection tally={[TALLY[0], { id: "enemy_1526_sfsui", count: 3, enemy: null }]} onFocusEnemy={noop} />
    </div>
);
