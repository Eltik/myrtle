import { itemIcon } from "#/components/operators/detail/impl/assets";
import type { IEnemy, IEnemyAttributes } from "#/lib/api/enemies";
import type { IEnemyDbRef, ILevel } from "#/lib/api/level";
import type { IMaterialItem } from "#/lib/api/materials";
import { getAvatarById } from "#/lib/utils";
import type { IStage, IZone } from "#/types/stages";
import { DROP_TYPE_META, OCC_FALLBACK, OCC_META } from "./constants";
import type { IDropGroup, IEnemyTally, IResolvedDrop, ISpawnRow, IStageEnemyStats } from "./types";

export function descToHtml(text: string): string {
    return text.replace(/<[@$][a-z0-9._]+>(.*?)<\/>/gi, (_m, inner: string) => `<strong style="color: var(--foreground)">${inner}</strong>`).replace(/\\n/g, "<br/>");
}

export function zoneLabel(zone: IZone | undefined, fallback: string): { title: string; subtitle: string | null } {
    if (!zone) return { title: fallback, subtitle: null };
    const title = zone.zoneNameSecond || zone.zoneNameFirst || zone.zoneNameTitleCurrent || zone.zoneId;
    const subtitle = zone.zoneNameFirst && zone.zoneNameFirst !== title ? zone.zoneNameFirst : null;
    return { title, subtitle };
}

export function tallyEnemies(level: ILevel | null, enemyData: Record<string, IEnemy>): IEnemyTally[] {
    if (!level) return [];
    const counts = new Map<string, number>();
    const refIds = new Set((level.enemyDbRefs ?? []).map((r) => r.id));
    for (const wave of level.waves ?? []) {
        for (const frag of wave.fragments ?? []) {
            for (const action of frag.actions ?? []) {
                const key = action.key;
                if (!key) continue;
                const isEnemy = action.actionType === "SPAWN" || refIds.has(key);
                if (!isEnemy) continue;
                counts.set(key, (counts.get(key) ?? 0) + (action.count ?? 1));
            }
        }
    }
    if (counts.size === 0) {
        for (const id of refIds) counts.set(id, 0);
    }
    return [...counts.entries()]
        .map(([id, count]) => ({ id, count, enemy: enemyData[id] ?? null }))
        .sort((a, b) => {
            const rank = (e: IEnemy | null) => (e?.enemyLevel === "BOSS" ? 0 : e?.enemyLevel === "ELITE" ? 1 : 2);
            return rank(a.enemy) - rank(b.enemy) || b.count - a.count;
        });
}

export function buildSpawnSchedule(level: ILevel | null, enemyData: Record<string, IEnemy>): { rows: ISpawnRow[]; hiddenGroups: string[] } {
    if (!level) return { rows: [], hiddenGroups: [] };
    const refIds = new Set((level.enemyDbRefs ?? []).map((r) => r.id));
    const rows: ISpawnRow[] = [];
    const hidden = new Set<string>();
    let idx = 0;
    let waveStart = 0;
    (level.waves ?? []).forEach((wave, wi) => {
        let fragTime = waveStart + (wave.preDelay ?? 0);
        for (const frag of wave.fragments ?? []) {
            fragTime += frag.preDelay ?? 0;
            for (const action of frag.actions ?? []) {
                if (action.hiddenGroup) hidden.add(action.hiddenGroup);
                const key = action.key;
                const isEnemy = action.actionType === "SPAWN" || (!!key && refIds.has(key));
                if (!isEnemy || !key) continue;
                const count = action.count ?? 1;
                const interval = action.interval ?? 0;
                rows.push({
                    idx: idx++,
                    id: key,
                    enemy: enemyData[key] ?? null,
                    count,
                    interval,
                    preDelay: action.preDelay ?? 0,
                    time: fragTime + (action.preDelay ?? 0),
                    wave: wi + 1,
                    hiddenGroup: action.hiddenGroup ?? null,
                    actionType: action.actionType,
                });
            }
        }
        waveStart = fragTime + (wave.postDelay ?? 0);
    });
    return { rows, hiddenGroups: [...hidden] };
}

function applyAttributeOverrides(base: IEnemyAttributes, ref: IEnemyDbRef | undefined): { attrs: IEnemyAttributes; hasOverride: boolean } {
    const overrides = ref?.overwrittenData?.attributes;
    if (!overrides) return { attrs: base, hasOverride: false };
    const attrs: IEnemyAttributes = { ...base };
    let hasOverride = false;
    for (const [key, ov] of Object.entries(overrides)) {
        if (!ov?.m_defined || !(key in attrs)) continue;
        (attrs as unknown as Record<string, number | boolean>)[key] = ov.m_value;
        hasOverride = true;
    }
    return { attrs, hasOverride };
}

/**
 * Resolve an enemy's effective stats for a given stage: the phase the stage uses
 * (`enemyDbRefs[].level`), any per-stage attribute overrides, the move-speed
 * multiplier, and the attack interval derived from attack speed.
 */
export function computeStageEnemyStats(enemy: IEnemy | null, ref: IEnemyDbRef | undefined, moveMultiplier: number): IStageEnemyStats | null {
    const phases = enemy?.stats?.levels ?? [];
    if (phases.length === 0) return null;
    const levelIndex = Math.min(Math.max(ref?.level ?? 0, 0), phases.length - 1);
    const { attrs, hasOverride } = applyAttributeOverrides(phases[levelIndex].attributes, ref);
    const aspd = attrs.attackSpeed || 100;
    return {
        levelIndex,
        phaseCount: phases.length,
        maxHp: attrs.maxHp,
        atk: attrs.atk,
        def: attrs.def,
        res: attrs.magicResistance,
        moveSpeed: attrs.moveSpeed * moveMultiplier,
        attackInterval: aspd > 0 ? (attrs.baseAttackTime * 100) / aspd : attrs.baseAttackTime,
        hasOverride,
    };
}

/** Build an `enemyId -> stage-effective stats` lookup for every enemy the stage declares. */
export function buildStageEnemyStats(level: ILevel | null, enemyData: Record<string, IEnemy>): Record<string, IStageEnemyStats> {
    const out: Record<string, IStageEnemyStats> = {};
    if (!level) return out;
    const moveMultiplier = level.options?.moveMultiplier ?? 1;
    for (const ref of level.enemyDbRefs ?? []) {
        const stats = computeStageEnemyStats(enemyData[ref.id] ?? null, ref, moveMultiplier);
        if (stats) out[ref.id] = stats;
    }
    return out;
}

export function rarityNum(tier: string | null | undefined): number {
    if (!tier) return 1;
    const n = Number(String(tier).replace("TIER_", ""));
    return n >= 1 && n <= 6 ? n : 1;
}

export function prettyCharName(id: string): string {
    const stem = id.split("_").pop() ?? id;
    return stem.charAt(0).toUpperCase() + stem.slice(1);
}

export function groupDrops(stage: IStage, items: Record<string, IMaterialItem>): IDropGroup[] {
    const rewards = stage.stageDropInfo?.displayDetailRewards ?? [];
    if (rewards.length === 0) return [];
    const byType = new Map<string, IResolvedDrop[]>();

    for (const reward of rewards) {
        const isChar = reward.itemType === "CHAR";
        const mat = items[reward.id];
        const resolved: IResolvedDrop = {
            reward,
            name: isChar ? prettyCharName(reward.id) : (mat?.name ?? reward.id),
            iconUrl: isChar ? getAvatarById(reward.id) : itemIcon(reward.id, mat?.iconId, null),
            rarity: isChar ? 5 : rarityNum(mat?.rarity),
            isChar,
            occ: OCC_META[reward.occPercent] ?? OCC_FALLBACK,
        };
        const list = byType.get(reward.dropType);
        if (list) list.push(resolved);
        else byType.set(reward.dropType, [resolved]);
    }

    return [...byType.entries()]
        .map(([type, drops]) => {
            const meta = DROP_TYPE_META[type];
            return {
                type,
                label: meta?.label ?? type,
                order: meta?.order ?? 99,
                drops: drops.sort((a, b) => b.occ.level - a.occ.level || b.rarity - a.rarity),
            };
        })
        .sort((a, b) => a.order - b.order);
}
