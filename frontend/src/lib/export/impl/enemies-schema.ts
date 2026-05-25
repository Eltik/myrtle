import type { IEnemyView } from "#/components/enemies/list/impl/types";
import type { IExportField, IExportSchema } from "./types";

const FIELDS: IExportField<IEnemyView>[] = [
    { id: "enemyId", label: "enemyId", group: "Identity", defaultEnabled: true, accessor: (e) => e.enemyId },
    { id: "enemyIndex", label: "enemyIndex", group: "Identity", defaultEnabled: true, accessor: (e) => e.enemyIndex },
    { id: "name", label: "name", group: "Identity", defaultEnabled: true, accessor: (e) => e.name },
    { id: "sortId", label: "sortId", group: "Identity", accessor: (e) => e.sortId },

    { id: "enemyLevel", label: "threat", group: "Classification", defaultEnabled: true, accessor: (e) => e.enemyLevel },
    { id: "attackType", label: "attackType", group: "Classification", accessor: (e) => e.attackType },
    { id: "applyWay", label: "applyWay", group: "Classification", accessor: (e) => e.applyWay },
    { id: "damageType", label: "damageType", group: "Classification", defaultEnabled: true, accessor: (e) => e.damageType },
    { id: "race", label: "race", group: "Classification", accessor: (e) => e.race },
    { id: "enemyTags", label: "tags", group: "Classification", accessor: (e) => e.enemyTags },

    { id: "maxHp", label: "hp", group: "Stats (level 0)", defaultEnabled: true, accessor: (e) => e.flatStats.maxHp },
    { id: "atk", label: "atk", group: "Stats (level 0)", defaultEnabled: true, accessor: (e) => e.flatStats.atk },
    { id: "def", label: "def", group: "Stats (level 0)", defaultEnabled: true, accessor: (e) => e.flatStats.def },
    { id: "res", label: "res", group: "Stats (level 0)", defaultEnabled: true, accessor: (e) => e.flatStats.res },
    { id: "aspd", label: "attackSpeed", group: "Stats (level 0)", accessor: (e) => e.flatStats.aspd },
    { id: "ms", label: "moveSpeed", group: "Stats (level 0)", accessor: (e) => e.flatStats.ms },
    { id: "weight", label: "weight", group: "Stats (level 0)", accessor: (e) => e.flatStats.weight },
    { id: "baseAttackTime", label: "baseAttackTime", group: "Stats (level 0)", accessor: (e) => e.flatStats.baseAttackTime },
    { id: "hpRecoveryPerSec", label: "hpRecoveryPerSec", group: "Stats (level 0)", accessor: (e) => e.flatStats.hpRecoveryPerSec },

    { id: "imm_stun", label: "stunImmune", group: "Immunities", accessor: (e) => e.immunities.stun },
    { id: "imm_silence", label: "silenceImmune", group: "Immunities", accessor: (e) => e.immunities.silence },
    { id: "imm_sleep", label: "sleepImmune", group: "Immunities", accessor: (e) => e.immunities.sleep },
    { id: "imm_frozen", label: "frozenImmune", group: "Immunities", accessor: (e) => e.immunities.frozen },
    { id: "imm_levitate", label: "levitateImmune", group: "Immunities", accessor: (e) => e.immunities.levitate },

    { id: "description", label: "description", group: "Description", accessor: (e) => e.description },
    { id: "ability", label: "ability", group: "Description", accessor: (e) => e.ability },
    { id: "abilityList", label: "abilityList", group: "Description", accessor: (e) => e.abilityList },
    { id: "linkEnemies", label: "linkEnemies", group: "Description", accessor: (e) => e.linkEnemies },

    { id: "hideInHandbook", label: "hideInHandbook", group: "Flags", accessor: (e) => e.hideInHandbook },
    { id: "hideInStage", label: "hideInStage", group: "Flags", accessor: (e) => e.hideInStage },
    { id: "invisibleDetail", label: "invisibleDetail", group: "Flags", accessor: (e) => e.invisibleDetail },
    { id: "isInvalidKilled", label: "isInvalidKilled", group: "Flags", accessor: (e) => e.isInvalidKilled },

    { id: "portrait", label: "portrait", group: "Media", accessor: (e) => e.portrait },

    {
        id: "stats",
        label: "stats",
        group: "Advanced",
        accessor: (e) => e.stats,
    },
];

export const enemiesExportSchema: IExportSchema<IEnemyView> = {
    id: "enemies",
    itemName: "enemy",
    pluralName: "enemies",
    fields: FIELDS,
};
