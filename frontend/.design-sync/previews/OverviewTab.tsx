import { OverviewTab } from "frontend";

// Handbook rows straight off api.myrtle.moe /static/enemies. `abilityList` is a
// flat list where `textFormat: "TITLE"` opens a new trait group — that is how a
// multi-form boss gets its "First Form" / "Second Form" headings.
const base = {
    attackType: null,
    ability: null,
    isInvalidKilled: false,
    overrideKillCntInfos: null,
    hideInHandbook: false,
    hideInStage: true,
    linkEnemies: [],
    invisibleDetail: false,
    stats: null,
    portrait: null,
    immunities: { stun: false, silence: false, sleep: false, frozen: false, levitate: false },
    flatStats: { maxHp: 0, atk: 0, def: 0, res: 0, aspd: 100, ms: 0, weight: 0, baseAttackTime: 0, hpRecoveryPerSec: 0 },
    applyWay: "MELEE",
};

const buldro = {
    ...base,
    enemyId: "enemy_2082_skzdd",
    enemyIndex: "SPT",
    enemyTags: ["sarkaz"],
    sortId: 1418,
    name: "Buldrokkas'tee, Holy Gun-Knight",
    enemyLevel: "BOSS",
    damageType: ["PHYSIC"],
    race: "Sarkaz",
    description: "An Apostolic Knight who guards the gates to the Holy City. He believes in order and willingly serves the peace brought about by Law, but he has never abandoned his doubts. In time, he will become Kazdel's true 'Patriot.'",
    abilityList: [
        { text: "[Ritual of Exhortation] More likely to be attacked, retreated units have significantly increased Redeployment Time.", textFormat: "NORMAL" },
        { text: "[Ritual of Holy Guard] Physical and Arts damage dealt to Buldrokkas'tee, Holy Gun-Knight, by units outside a certain range is significantly reduced.", textFormat: "NORMAL" },
        { text: "First Form", textFormat: "TITLE" },
        { text: "[Ritual of Automaton] Periodically summons multiple Holy Automatons.", textFormat: "NORMAL" },
        { text: "Second Form", textFormat: "TITLE" },
        { text: "Attacks become ranged and deal Physical splash damage.", textFormat: "NORMAL" },
        { text: "[Ritual of Strafing] Locks on to all High Grounds on the field, strafing each one in sequence to deal Physical damage.", textFormat: "NORMAL" },
        { text: "Third Form", textFormat: "TITLE" },
        { text: "When your mind is Fractured, Buldrokkas'tee, Holy Gun-Knight will enter Third Form, gaining increased ATK and summoning several elite enemies.", textFormat: "NORMAL" },
    ],
};

const signora = {
    ...base,
    enemyId: "enemy_1560_cnvlap",
    enemyIndex: "LOM",
    enemyTags: [],
    sortId: 1356,
    name: "'La Signora del Carnevale'",
    enemyLevel: "BOSS",
    damageType: ["PHYSIC", "MAGIC"],
    race: null,
    description: "La Signora del Carnevale finally takes the stage. The raucous revelry started with her, and it is she who will lead it to its finale. An endless carnival, a decadent joke.",
    abilityList: [
        { text: "Unblockable; is treated as an aerial unit when standing atop the landship and can only be hit by anti-air attacks; uses Drones to deal Arts damage", textFormat: "NORMAL" },
        { text: "Takes greatly reduced Physical and Arts damage; this effect is temporarily lost if Lappland the Decadenza has defeated fewer units than you when Carnevale ends", textFormat: "NORMAL" },
        { text: "First Form", textFormat: "TITLE" },
        { text: "Immobile; attacks deal Arts damage", textFormat: "NORMAL" },
        { text: "[Balla coi Lupi] Drone attack range expands to the entire field, ATK decreases, locks onto a target and attacks continuously for a period of time", textFormat: "NORMAL" },
        { text: "Second Form", textFormat: "TITLE" },
        { text: "Attacks deal Physical damage to 2 targets simultaneously", textFormat: "NORMAL" },
        { text: "[Encore] Teleports to the next checkpoint and deals Physical damage to all surrounding allied and enemy units; teleport cooldown shortens during Carnevale", textFormat: "NORMAL" },
    ],
};

const lancer = {
    ...base,
    enemyId: "enemy_1072_dlancer_2",
    enemyIndex: "S10",
    enemyTags: ["sarkaz"],
    sortId: 137,
    name: "Sarkaz Lancer Leader",
    enemyLevel: "ELITE",
    damageType: ["PHYSIC"],
    race: "Sarkaz",
    description: "A Sarkaz, more threatening than a normal Sarkaz Lancer. After special training, they are capable of rushing forward with a long spear, building up speed. The faster they charge, the harder they strike anyone who might stand in their way.",
    abilityList: [{ text: "Gradually accelerates when moving, and deals additional damage on the first attack after being blocked based upon Movement Speed.", textFormat: "NORMAL" }],
};

const slug = {
    ...base,
    enemyId: "enemy_1007_slime",
    enemyIndex: "B1",
    enemyTags: ["infection"],
    sortId: 1,
    name: "Originium Slug",
    enemyLevel: "NORMAL",
    damageType: ["PHYSIC"],
    race: "Infected Creature",
    description: "An infected wild creature. Due to their low intelligence, they are easily controlled by enemy casters for use as cannon fodder during invasions.",
    abilityList: [],
};

export const BossTraits = () => (
    <div className="w-full max-w-2xl">
        <OverviewTab enemy={buldro} />
    </div>
);

export const MultiFormTraits = () => (
    <div className="w-full max-w-2xl">
        <OverviewTab enemy={signora} />
    </div>
);

export const EliteOverview = () => (
    <div className="w-full max-w-2xl">
        <OverviewTab enemy={lancer} />
    </div>
);

export const DescriptionOnly = () => (
    <div className="w-full max-w-2xl">
        <OverviewTab enemy={slug} />
    </div>
);
