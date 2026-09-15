import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.base.nerds.title": {
        text: "Stats for Nerds",
        description: "Label on the collapsed panel of raw numbers. A knowing nod to the same phrase in video players.",
    },
    "profile.base.nerds.noChange": {
        text: "no change",
        description: "Replaces a delta figure when the two numbers are equal. Lowercase on purpose.",
    },
    "profile.base.nerds.output": {
        text: "Output",
        description: "Section heading over what the base produces. Rendered uppercase by CSS.",
    },
    "profile.base.nerds.productionEfficiency": {
        text: "Production efficiency",
        description: "Row label: the combined speed bonus of the producing rooms.",
    },
    "profile.base.nerds.totalValue": {
        text: "Total value (LMD-eq)",
        description: "Row label: every output converted to one number. 'LMD-eq' means LMD-equivalent; 'LMD' is the in-game currency.",
    },
    "profile.base.nerds.power": {
        text: "Power",
        description: "Section heading over the base's electricity. Rendered uppercase by CSS.",
    },
    "profile.base.nerds.generated": {
        text: "Generated",
        description: "Row label: electricity the plants make.",
    },
    "profile.base.nerds.consumed": {
        text: "Consumed",
        description: "Row label: electricity the rooms draw.",
    },
    "profile.base.nerds.net": {
        text: "Net",
        description: "Row label: generated electricity minus consumed.",
    },
    "profile.base.nerds.dormitories": {
        text: "Dormitories",
        description: "Section heading over the base's dormitories. Rendered uppercase by CSS.",
    },
    "profile.base.nerds.rooms": {
        text: "Rooms",
        description: "Row label: how many dormitories are built.",
    },
    "profile.base.nerds.summedLevels": {
        text: "Summed levels",
        description: "Row label: the dormitory levels added together.",
    },
    "profile.base.nerds.beds": {
        text: "Beds",
        description: "Row label: how many operators the dormitories hold at once.",
    },
    "profile.base.nerds.simRecovery": {
        text: "Sim recovery / hour",
        description: "Row label: morale the simulation restores per hour. 'Sim' is short for simulated.",
    },
    "profile.base.nerds.crew": {
        text: "Crew",
        description: "Section heading over the stationed operators. Rendered uppercase by CSS.",
    },
    "profile.base.nerds.stationed": {
        text: "Stationed",
        description: "Row label: how many operators are at work in the base.",
    },
    "profile.base.nerds.neverDeplete": {
        text: "Never deplete",
        description: "Row label: how many stationed operators never run out of morale.",
    },
    "profile.base.nerds.shortestMorale": {
        text: "Shortest morale",
        description: "Row label: how long the fastest-draining operator lasts.",
    },
    "profile.base.nerds.hours": {
        text: "{hours}h",
        description: "A duration in whole hours. 'h' is the abbreviation; very little room.",
    },
    "profile.base.nerds.days": {
        text: "{days}d",
        description: "A duration in whole days. 'd' is the abbreviation; very little room.",
    },
    "profile.base.nerds.sustained": {
        text: "Run 24/7 (Fiammetta)",
        description: "Row label: the operators the rotation never swaps out. 'Fiammetta' is the operator whose base skill makes it possible; keep the game's name.",
    },
    "profile.base.nerds.comparison": {
        text: "Optimized vs your base",
        description: "Section heading over the before-and-after figures. Rendered uppercase by CSS.",
    },
    "profile.base.nerds.roomsRestaffed": {
        text: "Rooms restaffed",
        description: "Row label: how many rooms the optimized plan changes.",
    },
    "profile.base.nerds.rotation": {
        text: "Rotation",
        description: "Section heading over the shift rotation's simulation results. Rendered uppercase by CSS.",
    },
    "profile.base.nerds.verdict": {
        text: "Verdict",
        description: "Row label: whether the rotation holds up over the simulated stretch.",
    },
    "profile.base.nerds.simulated": {
        text: "Simulated",
        description: "Row label: how long the rotation was simulated for.",
    },
    "profile.base.nerds.depleting": {
        text: "Operators depleting",
        description: "Row label: how many operators run out of morale during the simulation.",
    },
    "profile.base.nerds.dormOverflow": {
        text: "Dorm overflow at peak",
        description: "Row label: how many operators cannot get a bed at the busiest moment. 'Dorm' is short for dormitory.",
    },
    "profile.base.nerds.dormList": {
        text: "Dormitories - best first, game rates",
        description: "Section heading over the per-dormitory list. 'Game rates' means the figures use the game's own recovery numbers. Rendered uppercase by CSS.",
    },
    "profile.base.nerds.dormLevel": {
        text: "Lv {level}",
        description: "One dormitory's level in that list. 'Lv' is the game's abbreviation.",
    },
    "profile.base.nerds.dormBeds": {
        text: "{count} beds",
        description: "One dormitory's capacity in that list. Always in this one form; at most five.",
    },
    "profile.base.nerds.dormRecovery": {
        text: "{rate}/h",
        description: "One dormitory's morale recovery per hour. 'h' is the abbreviation for hour.",
    },
    "profile.base.nerds.dormAura": {
        text: "+{rate} aura",
        description: "Bonus recovery a resident's base skill gives everyone in the dormitory. 'Aura' is the community's word for a room-wide effect.",
    },
    "profile.base.nerds.dormSingle": {
        text: "+{rate} single",
        description: "Bonus recovery a resident's base skill gives one operator only.",
    },
    "profile.base.nerds.moraleOverTime": {
        text: "Morale over time",
        description: "Section heading over the per-operator morale lines. Rendered uppercase by CSS.",
    },
    "profile.base.nerds.perRoomChanges": {
        text: "Per-room changes",
        description: "Section heading over the rooms the optimized plan restaffs. Rendered uppercase by CSS.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
