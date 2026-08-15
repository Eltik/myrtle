import { SkillComparisonRow } from "frontend";

// One row of the Skills tab's mastery table. It diffs against the previous level
// it is given: SP cost, initial SP and duration tiles light up when they move,
// and `showDifferencesOnly` swaps the description for a "before -> after" list.
//
// Fixture is Młynar's third skill, Lv.7 through M3, from
// `/api/operators/char_4064_mlynar`.
const UNBRILLIANT_GLORY = [
    {
        "name": "Unbrilliant Glory",
        "rangeId": "3-18",
        "description": "Attack range <@ba.vup>increased</>, Trait effect increased by <@ba.vup>{trait_up}</>x (trait multiplier <@ba.vup>{per_kill_reduce:0%}</> for each enemy defeated), attacks hit <@ba.vup>{attack@max_target}</> targets for <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage. When any enemy within range is attacked by a Kazimierz Operator, deal an extra <@ba.vup>{atk_scale:0%}</> of Młynar ATK as <@ba.vup>True</> damage.",
        "skillType": "MANUAL",
        "durationType": "NONE",
        "spData": {
            "spType": "INCREASE_WITH_TIME",
            "maxChargeTime": 1,
            "spCost": 45,
            "initSp": 20,
            "increment": 1
        },
        "duration": 26,
        "blackboard": [
            {
                "key": "attack@atk_scale",
                "value": 1.5,
                "valueStr": null
            },
            {
                "key": "atk_scale",
                "value": 0.10999999940395357,
                "valueStr": null
            },
            {
                "key": "attack@max_target",
                "value": 5,
                "valueStr": null
            },
            {
                "key": "trait_up",
                "value": 2,
                "valueStr": null
            },
            {
                "key": "per_kill_reduce",
                "value": -0.10000000149011612,
                "valueStr": null
            }
        ]
    },
    {
        "name": "Unbrilliant Glory",
        "rangeId": "3-18",
        "description": "Attack range <@ba.vup>increased</>, Trait effect increased by <@ba.vup>{trait_up}</>x (trait multiplier <@ba.vup>{per_kill_reduce:0%}</> for each enemy defeated), attacks hit <@ba.vup>{attack@max_target}</> targets for <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage. When any enemy within range is attacked by a Kazimierz Operator, deal an extra <@ba.vup>{atk_scale:0%}</> of Młynar ATK as <@ba.vup>True</> damage.",
        "skillType": "MANUAL",
        "durationType": "NONE",
        "spData": {
            "spType": "INCREASE_WITH_TIME",
            "maxChargeTime": 1,
            "spCost": 45,
            "initSp": 20,
            "increment": 1
        },
        "duration": 27,
        "blackboard": [
            {
                "key": "attack@atk_scale",
                "value": 1.600000023841858,
                "valueStr": null
            },
            {
                "key": "atk_scale",
                "value": 0.10999999940395357,
                "valueStr": null
            },
            {
                "key": "attack@max_target",
                "value": 5,
                "valueStr": null
            },
            {
                "key": "trait_up",
                "value": 2,
                "valueStr": null
            },
            {
                "key": "per_kill_reduce",
                "value": -0.10000000149011612,
                "valueStr": null
            }
        ]
    },
    {
        "name": "Unbrilliant Glory",
        "rangeId": "3-18",
        "description": "Attack range <@ba.vup>increased</>, Trait effect increased by <@ba.vup>{trait_up}</>x (trait multiplier <@ba.vup>{per_kill_reduce:0%}</> for each enemy defeated), attacks hit <@ba.vup>{attack@max_target}</> targets for <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage. When any enemy within range is attacked by a Kazimierz Operator, deal an extra <@ba.vup>{atk_scale:0%}</> of Młynar ATK as <@ba.vup>True</> damage.",
        "skillType": "MANUAL",
        "durationType": "NONE",
        "spData": {
            "spType": "INCREASE_WITH_TIME",
            "maxChargeTime": 1,
            "spCost": 45,
            "initSp": 20,
            "increment": 1
        },
        "duration": 27,
        "blackboard": [
            {
                "key": "attack@atk_scale",
                "value": 1.7000000476837158,
                "valueStr": null
            },
            {
                "key": "atk_scale",
                "value": 0.10999999940395357,
                "valueStr": null
            },
            {
                "key": "attack@max_target",
                "value": 5,
                "valueStr": null
            },
            {
                "key": "trait_up",
                "value": 2,
                "valueStr": null
            },
            {
                "key": "per_kill_reduce",
                "value": -0.10000000149011612,
                "valueStr": null
            }
        ]
    },
    {
        "name": "Unbrilliant Glory",
        "rangeId": "3-18",
        "description": "Attack range <@ba.vup>increased</>, Trait effect increased by <@ba.vup>{trait_up}</>x (trait multiplier <@ba.vup>{per_kill_reduce:0%}</> for each enemy defeated), attacks hit <@ba.vup>{attack@max_target}</> targets for <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage. When any enemy within range is attacked by a Kazimierz Operator, deal an extra <@ba.vup>{atk_scale:0%}</> of Młynar ATK as <@ba.vup>True</> damage.",
        "skillType": "MANUAL",
        "durationType": "NONE",
        "spData": {
            "spType": "INCREASE_WITH_TIME",
            "maxChargeTime": 1,
            "spCost": 42,
            "initSp": 20,
            "increment": 1
        },
        "duration": 28,
        "blackboard": [
            {
                "key": "attack@atk_scale",
                "value": 1.7999999523162842,
                "valueStr": null
            },
            {
                "key": "atk_scale",
                "value": 0.119999997317791,
                "valueStr": null
            },
            {
                "key": "attack@max_target",
                "value": 5,
                "valueStr": null
            },
            {
                "key": "trait_up",
                "value": 2,
                "valueStr": null
            },
            {
                "key": "per_kill_reduce",
                "value": -0.10000000149011612,
                "valueStr": null
            }
        ]
    }
];

const [LV7, M1, M2, M3] = UNBRILLIANT_GLORY;

export const MasteryLadder = () => (
    <div className="flex flex-col">
        <SkillComparisonRow isFirst levelData={LV7} levelIndex={6} prevLevelData={null} />
        <SkillComparisonRow levelData={M1} levelIndex={7} prevLevelData={LV7} />
        <SkillComparisonRow levelData={M2} levelIndex={8} prevLevelData={M1} />
        <SkillComparisonRow isLast levelData={M3} levelIndex={9} prevLevelData={M2} />
    </div>
);

export const SingleRow = () => (
    <div className="flex flex-col">
        <SkillComparisonRow isFirst isLast levelData={M3} levelIndex={9} prevLevelData={M2} />
    </div>
);

export const DifferencesOnly = () => (
    <div className="flex flex-col">
        <SkillComparisonRow isFirst levelData={LV7} levelIndex={6} prevLevelData={null} showDifferencesOnly />
        <SkillComparisonRow levelData={M1} levelIndex={7} prevLevelData={LV7} showDifferencesOnly />
        <SkillComparisonRow isLast levelData={M3} levelIndex={9} prevLevelData={M2} showDifferencesOnly />
    </div>
);
