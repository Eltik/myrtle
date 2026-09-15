import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The taxonomies in `constants.ts` are plain data in a module with no React,
 * so they carry message KEYS and whichever component renders a row resolves it
 * with `t()`.
 *
 * The raw game tokens those tables are keyed BY (`HOME_PLACE`,
 * `ON_SKILL_START`, the English category ids) are not messages: they are the
 * data's own identifiers and must not move.
 */
export const namespace = "operators";

export const messages = {
    "tab.info": {
        text: "Information",
        description: "Operator-detail tab: stats, profile, talents and modules.",
    },
    "tab.skills": {
        text: "Skills",
        description: "Operator-detail tab: the operator's active skills. The skill names themselves come from the game data.",
    },
    "tab.levelup": {
        text: "Level-Up Cost",
        description: "Operator-detail tab: the materials needed to promote and upgrade.",
    },
    "tab.skins": {
        text: "Skins",
        description: "Operator-detail tab: alternate outfits. 'Skin' is the game's word for a cosmetic outfit.",
    },
    "tab.audio": {
        text: "Audio/SFX",
        description: "Operator-detail tab: voice lines and battle sound effects. 'SFX' is the usual shorthand for sound effects.",
    },
    "tab.lore": {
        text: "Lore",
        description: "Operator-detail tab: archive files and story text.",
    },
    "voice.lang.jp": {
        text: "Japanese",
        description: "Voice-over language.",
    },
    "voice.lang.cnMandarin": {
        text: "Mandarin",
        description: "Voice-over language: Mandarin Chinese.",
    },
    "voice.lang.en": {
        text: "English",
        description: "Voice-over language.",
    },
    "voice.lang.kr": {
        text: "Korean",
        description: "Voice-over language.",
    },
    "voice.lang.cnTopolect": {
        text: "Cantonese",
        description: "Voice-over language: the game ships a handful of Cantonese tracks.",
    },
    "voice.lang.ger": {
        text: "German",
        description: "Voice-over language.",
    },
    "voice.lang.ita": {
        text: "Italian",
        description: "Voice-over language.",
    },
    "voice.lang.rus": {
        text: "Russian",
        description: "Voice-over language.",
    },
    "voice.lang.fre": {
        text: "French",
        description: "Voice-over language.",
    },
    "voice.lang.spa": {
        text: "Spanish",
        description: "Voice-over language.",
    },
    "voice.lang.linkage": {
        text: "Linkage",
        description: "Voice-over 'language' for duet lines an operator shares with another character, which the game files store as their own track.",
    },
    "voice.lang.short.jp": {
        text: "JP",
        description: "Two-letter chip for the Japanese track. Fixed-width button; two or three characters at most.",
    },
    "voice.lang.short.cnMandarin": {
        text: "CN",
        description: "Two-letter chip for the Mandarin track. Fixed-width button; two or three characters at most.",
    },
    "voice.lang.short.en": {
        text: "EN",
        description: "Two-letter chip for the English track. Fixed-width button; two or three characters at most.",
    },
    "voice.lang.short.kr": {
        text: "KR",
        description: "Two-letter chip for the Korean track. Fixed-width button; two or three characters at most.",
    },
    "voice.lang.short.cnTopolect": {
        text: "粤",
        description: "Chip for the Cantonese track - the Chinese character for Yue/Cantonese, as the game itself labels it. Fixed-width button.",
    },
    "voice.lang.short.ger": {
        text: "DE",
        description: "Two-letter chip for the German track. Fixed-width button; two or three characters at most.",
    },
    "voice.lang.short.ita": {
        text: "IT",
        description: "Two-letter chip for the Italian track. Fixed-width button; two or three characters at most.",
    },
    "voice.lang.short.rus": {
        text: "RU",
        description: "Two-letter chip for the Russian track. Fixed-width button; two or three characters at most.",
    },
    "voice.lang.short.fre": {
        text: "FR",
        description: "Two-letter chip for the French track. Fixed-width button; two or three characters at most.",
    },
    "voice.lang.short.spa": {
        text: "ES",
        description: "Two-letter chip for the Spanish track. Fixed-width button; two or three characters at most.",
    },
    "voice.lang.short.linkage": {
        text: "LINK",
        description: "Chip for duet lines shared with another character. Fixed-width button; four characters at most.",
    },
    "voice.category.greetings": {
        text: "Greetings",
        description: "Voice-line category: lines the operator says when you open the app or tap them.",
    },
    "voice.category.conversations": {
        text: "Conversations",
        description: "Voice-line category: squad chatter.",
    },
    "voice.category.trust": {
        text: "Trust",
        description: "Voice-line category: lines unlocked by raising the operator's trust. 'Trust' is the game's own affinity stat.",
    },
    "voice.category.promotions": {
        text: "Promotions",
        description: "Voice-line category: lines for levelling up and for each Elite promotion.",
    },
    "voice.category.battle": {
        text: "Battle",
        description: "Voice-line category: in-combat lines.",
    },
    "voice.category.idle": {
        text: "Idle",
        description: "Voice-line category: lines said when the operator is left alone.",
    },
    "voice.category.dorm": {
        text: "Dorm",
        description: "Voice-line category: lines said in the base. 'Dorm' is short for dormitory, the base facility.",
    },
    "voice.category.special": {
        text: "Special",
        description: "Voice-line category: birthdays, new year, anniversary and recruitment lines.",
    },
    "voice.category.other": {
        text: "Other",
        description: "Voice-line category: the catch-all bucket.",
    },
    "sfx.category.deploy": {
        text: "Deploy",
        description: "Battle-audio category: the sound of putting the operator on the map. Tab label, keep it short.",
    },
    "sfx.category.attack": {
        text: "Attack",
        description: "Battle-audio category: normal attack sounds. Tab label, keep it short.",
    },
    "sfx.category.skill": {
        text: "Skill",
        description: "Battle-audio category: skill sounds. Tab label, keep it short.",
    },
    "sfx.category.voice": {
        text: "Voice",
        description: "Battle-audio category: in-combat voice barks. Tab label, keep it short.",
    },
    "sfx.category.other": {
        text: "Other",
        description: "Battle-audio category: the catch-all bucket. Tab label, keep it short.",
    },
    "sfx.event.unitBorn": {
        text: "Deployment",
        description: "Battle-audio event: the operator is placed on the map.",
    },
    "sfx.event.abilityStart": {
        text: "Attack",
        description: "Battle-audio event: an attack begins.",
    },
    "sfx.event.abilityHit": {
        text: "Attack Impact",
        description: "Battle-audio event: an attack lands.",
    },
    "sfx.event.abilityOn": {
        text: "Attack (Active)",
        description: "Battle-audio event: the looping sound while an attack is ongoing.",
    },
    "sfx.event.abilityEnd": {
        text: "Attack End",
        description: "Battle-audio event: an attack finishes.",
    },
    "sfx.event.abilityAttackFinish": {
        text: "Attack Finish",
        description: "Battle-audio event: the tail of an attack animation.",
    },
    "sfx.event.abilityCheckPoint": {
        text: "Attack Checkpoint",
        description: "Battle-audio event: a mid-animation cue inside an attack.",
    },
    "sfx.event.skillStart": {
        text: "Skill Activation",
        description: "Battle-audio event: a skill is switched on.",
    },
    "sfx.event.skillChantStart": {
        text: "Skill Charge",
        description: "Battle-audio event: a skill's wind-up.",
    },
    "sfx.event.skillSpecialPoint": {
        text: "Skill Trigger",
        description: "Battle-audio event: a cue partway through a skill.",
    },
    "sfx.event.skillFinish": {
        text: "Skill End",
        description: "Battle-audio event: a skill ends.",
    },
    "sfx.event.skillOn": {
        text: "Skill Ready",
        description: "Battle-audio event: the skill has charged and can be used.",
    },
    "sfx.event.skillFailed": {
        text: "Skill Failed",
        description: "Battle-audio event: a skill could not fire.",
    },
    "sfx.event.unitDead": {
        text: "Defeated",
        description: "Battle-audio event: the operator is knocked out.",
    },
    "sfx.event.spineEventTrigger": {
        text: "Animation Cue",
        description: "Battle-audio event: a sound attached to a frame of the animation. 'Spine' is the animation runtime and is dropped from the label on purpose.",
    },
    "sfx.event.customTrigger": {
        text: "Custom Trigger",
        description: "Battle-audio event: a one-off cue the game data defines itself.",
    },
    "stat.health": {
        text: "Health",
        description: "Combat stat row label, beside the HP icon. Shared by the operator table and the summon table.",
    },
    "stat.defense": {
        text: "Defense",
        description: "Combat stat row label, beside the DEF icon: physical defense.",
    },
    "stat.artsResistance": {
        text: "Arts Resistance",
        description: "Combat stat row label, beside the RES icon. 'Arts' is the game's word for magic, so keep the game's own term.",
    },
    "stat.redeployTime": {
        text: "Redeploy Time",
        description: "Combat stat row label: the cooldown before the operator can be placed again.",
    },
    "stat.attackPower": {
        text: "Attack Power",
        description: "Combat stat row label, beside the ATK icon.",
    },
    "stat.attackInterval": {
        text: "Attack Interval",
        description: "Combat stat row label: seconds between attacks.",
    },
    "stat.block": {
        text: "Block",
        description: "Combat stat row label: how many enemies the operator holds at once.",
    },
    "stat.dpCost": {
        text: "DP Cost",
        description: "Combat stat row label: deployment cost. 'DP' is the in-battle currency and stays as-is.",
    },
    "stat.seconds": {
        text: "{value} sec",
        description: "A stat value in seconds, e.g. '70 sec'. {value} is already formatted. Abbreviate the unit; the column is narrow.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on a registry/constants entry and resolved
// by the consuming component as `t(item.labelKey)`, so the extractor has no
// literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
