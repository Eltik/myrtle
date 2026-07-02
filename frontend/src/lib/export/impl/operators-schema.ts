import type { IOperatorExportRow } from "#/components/operators/list/impl/types";
import type { IExportField, IExportSchema } from "./types";

const FIELDS: IExportField<IOperatorExportRow>[] = [
    { id: "id", label: "id", group: "Identity", defaultEnabled: true, accessor: (o) => o.id },
    { id: "name", label: "name", group: "Identity", defaultEnabled: true, accessor: (o) => o.name },
    { id: "appellation", label: "appellation", group: "Identity", accessor: (o) => o.appellation },
    { id: "displayNumber", label: "displayNumber", group: "Identity", accessor: (o) => o.displayNumber },
    { id: "rarity", label: "rarity", group: "Identity", defaultEnabled: true, accessor: (o) => o.rarity },
    { id: "rarityTier", label: "rarityTier", group: "Identity", accessor: (o) => `TIER_${o.rarity}` },
    { id: "profession", label: "profession", group: "Class", defaultEnabled: true, accessor: (o) => o.profession },
    { id: "subProfessionId", label: "subProfession", group: "Class", defaultEnabled: true, accessor: (o) => o.subProfessionId },
    { id: "position", label: "position", group: "Class", accessor: (o) => o.position },
    { id: "tagList", label: "tags", group: "Class", accessor: (o) => o.tagList },

    { id: "nationId", label: "nation", group: "Origin", accessor: (o) => o.nationId },
    { id: "groupId", label: "faction", group: "Origin", accessor: (o) => o.groupId },
    { id: "teamId", label: "team", group: "Origin", accessor: (o) => o.teamId },
    { id: "gender", label: "gender", group: "Origin", accessor: (o) => o.gender },
    { id: "race", label: "race", group: "Origin", accessor: (o) => o.race },
    { id: "placeOfBirth", label: "placeOfBirth", group: "Origin", accessor: (o) => o.placeOfBirth },

    { id: "hp", label: "hp", group: "Stats (max)", defaultEnabled: true, accessor: (o) => o.stats?.hp ?? null },
    { id: "atk", label: "atk", group: "Stats (max)", defaultEnabled: true, accessor: (o) => o.stats?.atk ?? null },
    { id: "def", label: "def", group: "Stats (max)", defaultEnabled: true, accessor: (o) => o.stats?.def ?? null },
    { id: "res", label: "res", group: "Stats (max)", defaultEnabled: true, accessor: (o) => o.stats?.res ?? null },
    { id: "cost", label: "cost", group: "Stats (max)", accessor: (o) => o.stats?.cost ?? null },
    { id: "block", label: "block", group: "Stats (max)", accessor: (o) => o.stats?.block ?? null },

    { id: "voiceActors", label: "voiceActors", group: "Credits", accessor: (o) => o.voiceActors },
    { id: "artists", label: "artists", group: "Credits", accessor: (o) => o.artists },

    { id: "description", label: "description", group: "Description", accessor: (o) => o.description },
    { id: "itemUsage", label: "itemUsage", group: "Description", accessor: (o) => o.itemUsage },
    { id: "itemDesc", label: "itemDesc", group: "Description", accessor: (o) => o.itemDesc },
    { id: "itemObtainApproach", label: "itemObtainApproach", group: "Description", accessor: (o) => o.itemObtainApproach },

    { id: "isNotObtainable", label: "isNotObtainable", group: "Flags", accessor: (o) => o.isNotObtainable },
    { id: "isSpChar", label: "isSpChar", group: "Flags", accessor: (o) => o.isSpChar },
    { id: "maxPotentialLevel", label: "maxPotentialLevel", group: "Flags", accessor: (o) => o.maxPotentialLevel },
    { id: "hasNotes", label: "hasNotes", group: "Flags", accessor: (o) => o.hasNotes },

    { id: "portrait", label: "portrait", group: "Media", accessor: (o) => o.portrait },
    { id: "skin", label: "skin", group: "Media", accessor: (o) => o.skin },
];

export const operatorsExportSchema: IExportSchema<IOperatorExportRow> = {
    id: "operators",
    itemName: "operator",
    pluralName: "operators",
    fields: FIELDS,
};
