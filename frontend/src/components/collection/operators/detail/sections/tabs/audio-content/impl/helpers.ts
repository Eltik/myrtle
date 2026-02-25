import type { LangType, PlaceType, Voice, Voices } from "~/types/api/impl/voice";
import { LANGUAGE_LABELS } from "./constants";
import type { VoiceLine } from "./types";

export function sanitizeFilename(name: string): string {
    return name.replace(/[^a-zA-Z0-9\-_]/g, "_");
}

export function generateVoiceFilename(operatorName: string, voiceTitle: string, language: LangType, voiceUrl: string): string {
    const ext = voiceUrl.split(".").pop()?.split("?")[0] ?? "mp3";
    const langLabel = LANGUAGE_LABELS[language] ?? language;
    return `${sanitizeFilename(operatorName)}_${sanitizeFilename(voiceTitle)}_${sanitizeFilename(langLabel)}.${ext}`;
}

export async function downloadVoiceLine(audioUrl: string, filename: string): Promise<void> {
    const response = await fetch(audioUrl);
    const blob = await response.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}

// Map PlaceType to category name
export function getCategoryName(placeType: PlaceType): string {
    switch (placeType) {
        case "GREETING":
        case "NEW_YEAR":
        case "ANNIVERSARY":
        case "BIRTHDAY":
            return "Greetings";
        case "BATTLE_START":
        case "BATTLE_FACE_ENEMY":
        case "BATTLE_SELECT":
        case "BATTLE_PLACE":
        case "BATTLE_SKILL_1":
        case "BATTLE_SKILL_2":
        case "BATTLE_SKILL_3":
        case "BATTLE_SKILL_4":
        case "FOUR_STAR":
        case "THREE_STAR":
        case "TWO_STAR":
        case "LOSE":
            return "Combat";
        case "HOME_PLACE":
        case "HOME_SHOW":
        case "HOME_WAIT":
            return "Interaction";
        case "BUILDING_PLACE":
        case "BUILDING_TOUCHING":
        case "BUILDING_FAVOR_BUBBLE":
            return "Base";
        case "LEVEL_UP":
        case "EVOLVE_ONE":
        case "EVOLVE_TWO":
            return "Level Up";
        case "GACHA":
            return "Recruitment";
        case "SQUAD":
        case "SQUAD_FIRST":
            return "Squad";
        default:
            return "Other";
    }
}

export function formatVoices(voiceData: Voices | Voice[]): VoiceLine[] {
    const voiceLines: VoiceLine[] = [];

    // If we have actual voice data, use it
    if (Array.isArray(voiceData)) {
        for (const voice of voiceData) {
            voiceLines.push({
                id: voice.voiceId ?? voice.id ?? "",
                title: voice.voiceTitle ?? "Voice Line",
                text: voice.voiceText ?? "",
                data: voice.data ?? undefined,
                languages: voice.languages ?? undefined,
                placeType: voice.placeType ?? undefined,
            });
        }
    } else if (voiceData && typeof voiceData === "object") {
        // Handle Voices object format (with charWords map)
        const voices = "charWords" in voiceData ? voiceData.charWords : voiceData;

        for (const [_key, voice] of Object.entries(voices)) {
            if (voice && typeof voice === "object" && "voiceId" in voice) {
                const v = voice as Voice;
                voiceLines.push({
                    id: v.voiceId ?? v.id ?? _key,
                    title: v.voiceTitle ?? "Voice Line",
                    text: v.voiceText ?? "",
                    data: v.data ?? undefined,
                    languages: v.languages ?? undefined,
                    placeType: v.placeType ?? undefined,
                });
            }
        }
    }

    // Sort by voice index if available (CN_001, CN_002, etc.)
    voiceLines.sort((a, b) => {
        const numA = Number.parseInt(a.id.replace(/\D/g, ""), 10) || 0;
        const numB = Number.parseInt(b.id.replace(/\D/g, ""), 10) || 0;
        return numA - numB;
    });

    return voiceLines;
}
