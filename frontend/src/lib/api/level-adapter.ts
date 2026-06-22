/**
 * Adapts the wire `IStageMap` (+ the enemy handbook) into the model the Stage
 * Viewer board, scrub-simulator, and enemy panel render. Pure + memo-friendly:
 * given the same inputs it returns structurally identical output.
 */
import type { IEnemy, IEnemyDamageType, IEnemyHandbook, IEnemyLevel } from "./enemies";
import type { IHiddenRoute, IMapOptions, IModifier, IPoint, IRosterEntry, IScheduleEntry, IStageMap, IWaveMarker } from "./level";

/** Pixels per tile on the board. */
export const TILE = 52;

export type DeployKind = "MELEE" | "RANGED" | "NONE";

export interface IBoardCell {
    x: number;
    y: number;
    kind: string;
    passable: boolean;
    /** Derived from `buildable`: where an operator may deploy. */
    deployable: DeployKind;
    heightType: string;
}

export interface IBoardRoute {
    motion: string;
    /** Polyline in tile coords (x = col, y = row, top row = 0). */
    points: IPoint[];
}

export interface IBoardEnemyStats {
    hp: number;
    atk: number;
    def: number;
    res: number;
    moveSpeed: number;
    atkTime: number;
    atkRange: string;
    hpRecover: number;
    mass: string;
}

/** A roster/schedule enemy joined against the handbook for the detail panel. */
export interface IBoardEnemy {
    enemyId: string;
    name: string;
    enemyIndex: string;
    level: IEnemyLevel;
    race: string | null;
    damageType: IEnemyDamageType[];
    applyWay: string;
    motion: string;
    trait: string;
    stats: IBoardEnemyStats | null;
}

export interface IBoardModel {
    stageId: string;
    code: string;
    name: string | null;
    width: number;
    height: number;
    tileSize: number;
    cells: IBoardCell[];
    grid: Record<string, IBoardCell>;
    routes: IBoardRoute[];
    hiddenRoutes: IHiddenRoute[];
    schedule: IScheduleEntry[];
    waves: IWaveMarker[];
    roster: IRosterEntry[];
    enemies: Record<string, IBoardEnemy>;
    options: IMapOptions;
    modifiers: IModifier[];
    duration: number;
}

export const tileKey = (x: number, y: number): string => `${x},${y}`;

/** Arknights rich-text tags (`<@ba.vup>…</>`, `<color=…>…</color>`) → plain text. */
function stripTags(s: string | null | undefined): string {
    if (!s) return "";
    return s
        .replace(/<[^>]+>/g, "")
        .replace(/\\n/g, " ")
        .replace(/\s+/g, " ")
        .trim();
}

function massLabel(level: number): string {
    if (level <= 0) return "-";
    if (level >= 3) return "High";
    if (level === 2) return "Normal";
    return "Low";
}

function buildableToDeploy(buildable: string): DeployKind {
    const b = buildable.toUpperCase();
    if (b === "MELEE") return "MELEE";
    if (b === "RANGED") return "RANGED";
    return "NONE";
}

/** Join a single handbook enemy into the panel's flat shape. */
export function joinEnemy(enemyId: string, handbook: IEnemyHandbook | undefined): IBoardEnemy {
    const e: IEnemy | undefined = handbook?.enemyData[enemyId];
    const lvl0 = e?.stats?.levels[0];
    const a = lvl0?.attributes;
    const race = e?.enemyTags?.[0] ?? null;
    const trait = stripTags(e?.description) || stripTags(e?.abilityList?.[0]?.text) || "No description available.";
    const applyWay = lvl0?.applyWay ?? e?.attackType ?? "-";

    return {
        enemyId,
        name: e?.name ?? enemyId,
        enemyIndex: e?.enemyIndex ?? "-",
        level: e?.enemyLevel ?? "NORMAL",
        race,
        damageType: e?.damageType ?? [],
        applyWay,
        motion: lvl0?.motion ?? "WALK",
        trait,
        stats: a
            ? {
                  hp: a.maxHp,
                  atk: a.atk,
                  def: a.def,
                  res: a.magicResistance,
                  moveSpeed: a.moveSpeed,
                  atkTime: a.baseAttackTime,
                  atkRange: lvl0?.rangeRadius != null && lvl0.rangeRadius > 0 ? `${lvl0.rangeRadius}` : e?.attackType === "MELEE" ? "Melee" : "-",
                  hpRecover: a.hpRecoveryPerSec,
                  mass: massLabel(a.massLevel),
              }
            : null,
    };
}

/** Build the full board model from a wire stage map + the enemy handbook. */
export function toBoardModel(map: IStageMap, handbook: IEnemyHandbook | undefined): IBoardModel {
    const cells: IBoardCell[] = [];
    const grid: Record<string, IBoardCell> = {};
    for (let y = 0; y < map.tiles.length; y++) {
        const row = map.tiles[y];
        if (!row) continue;
        for (let x = 0; x < row.length; x++) {
            const t = row[x];
            if (!t) continue;
            const cell: IBoardCell = {
                x,
                y,
                kind: t.kind,
                passable: t.passable,
                deployable: buildableToDeploy(t.buildable),
                heightType: t.heightType,
            };
            cells.push(cell);
            grid[tileKey(x, y)] = cell;
        }
    }

    // Join every enemy that appears (schedule + roster) once.
    const enemies: Record<string, IBoardEnemy> = {};
    const ids = new Set<string>([...map.schedule.map((s) => s.enemyId), ...map.roster.map((r) => r.enemyId)]);
    for (const id of ids) enemies[id] = joinEnemy(id, handbook);

    return {
        stageId: map.stageId,
        code: map.code,
        name: map.name,
        width: map.width,
        height: map.height,
        tileSize: TILE,
        cells,
        grid,
        routes: map.routes.map((r) => ({ motion: r.motion, points: r.points })),
        hiddenRoutes: map.hiddenRoutes ?? [],
        schedule: map.schedule ?? [],
        waves: map.waves ?? [],
        roster: map.roster,
        enemies,
        options: map.options,
        modifiers: map.modifiers ?? [],
        duration: map.duration,
    };
}
