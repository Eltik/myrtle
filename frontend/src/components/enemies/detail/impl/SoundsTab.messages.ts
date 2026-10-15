import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "enemies";

export const messages = {
    "sounds.intro": {
        text: "{count, plural, one {# sound} other {# sounds}} pulled from the game's battle audio.",
        description: "Line above the enemy's sound list. {count} is how many sound rows follow.",
    },
    "sounds.category.spawn": {
        text: "Spawn",
        description: "Heading over the sounds an enemy makes when it appears on the map.",
    },
    "sounds.category.attack": {
        text: "Basic attack",
        description: "Heading over the sounds of the enemy's normal attack.",
    },
    "sounds.category.skill": {
        text: "Skills",
        description: "Heading over the sounds of the enemy's skills and special effects.",
    },
    "sounds.category.projectile": {
        text: "Projectiles",
        description: "Heading over the sounds of things the enemy shoots or throws.",
    },
    "sounds.category.death": {
        text: "Death",
        description: "Heading over the sounds an enemy makes when it is defeated.",
    },
    "sounds.category.other": {
        text: "Other",
        description: "Heading over sounds that fit no other group, like animation cues.",
    },
    "sounds.projectileNote": {
        text: "Matched by name, so the same projectile can show up on related enemies.",
        description: "Small note under the Projectiles heading. Projectile sounds are linked to an enemy by a shared code name, not by a direct reference, so they may also appear on the enemy's variants.",
    },
    "sounds.event.abilityStart": {
        text: "Wind-up",
        description: "Row label: the sound that plays as an attack or skill starts.",
    },
    "sounds.event.abilityOn": {
        text: "Release",
        description: "Row label: the sound that plays the moment an attack or skill goes off.",
    },
    "sounds.event.abilityHit": {
        text: "Impact",
        description: "Row label: the sound that plays when an attack or skill lands on its target.",
    },
    "sounds.event.abilityEnd": {
        text: "End",
        description: "Row label: the sound that plays as an attack or skill ends.",
    },
    "sounds.event.abilityAttackFinish": {
        text: "Finish",
        description: "Row label: the sound that plays when an attack sequence completes.",
    },
    "sounds.event.unitBorn": {
        text: "Appears",
        description: "Row label: the sound that plays when the enemy enters the map.",
    },
    "sounds.event.bossEnter": {
        text: "Boss entrance",
        description: "Row label: the sound that plays when a boss makes its entrance.",
    },
    "sounds.event.unitDead": {
        text: "Defeated",
        description: "Row label: the sound that plays when the enemy is defeated.",
    },
    "sounds.event.buffStart": {
        text: "Effect",
        description: "Row label: the sound that plays when one of the enemy's special effects kicks in.",
    },
    "sounds.event.projectileHit": {
        text: "Hit",
        description: "Row label: the sound that plays when a projectile hits something.",
    },
    "sounds.event.projectileReach": {
        text: "Lands",
        description: "Row label: the sound that plays when a projectile reaches its target spot.",
    },
    "sounds.event.spineEventTrigger": {
        text: "Animation cue",
        description: "Row label: a sound tied to a moment in the enemy's animation.",
    },
    "sounds.event.customTrigger": {
        text: "Scripted",
        description: "Row label: a sound the game fires from a scripted moment rather than an attack.",
    },
    "sounds.play": {
        text: "Play",
        description: "Accessible name of the play button on a sound row.",
    },
    "sounds.pause": {
        text: "Pause",
        description: "Accessible name of the play button while that sound is playing.",
    },
    "sounds.unavailable": {
        text: "Unavailable",
        description: "Accessible name of the play button, and the badge on the row, when the sound file cannot be loaded.",
    },
    "sounds.playVariant": {
        text: "Play take {index}",
        description: "Accessible name of one numbered button when the game ships several interchangeable takes of a sound. {index} counts from 1.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
