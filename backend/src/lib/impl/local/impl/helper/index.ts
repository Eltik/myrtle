import type { HandbookItem, OperatorProfile } from "../../../../../types/impl/lib/impl/local/impl/gamedata/impl/handbook";

/**
 * @author All credit to https://github.com/Awedtan/HellaBot/
 * @param text
 * @returns string
 */

export function removeStyleTags(text: string): string {
    return text.replace(/<.[a-z]{2,5}?\.[^<]+>|<\/[^<]*>|<color=[^>]+>/g, "");
}

export function parseOperatorProfile(storyTextAudio: HandbookItem["storyTextAudio"]): OperatorProfile {
    const profile: OperatorProfile = {
        basicInfo: {
            codeName: "",
            combatExperience: "",
            dateOfBirth: "",
            gender: "",
            height: "",
            infectionStatus: "",
            placeOfBirth: "",
            race: "",
        },
        physicalExam: {
            combatSkill: "",
            mobility: "",
            originiumArtsAssimilation: "",
            physicalResilience: "",
            physicalStrength: "",
            tacticalAcumen: "",
        },
    };

    const basicInfo = storyTextAudio[0]?.stories[0]?.storyText ?? "";
    const physicalInfo = storyTextAudio[1]?.stories[0]?.storyText ?? "";

    const infoLines = basicInfo.split("\n");
    const physicalLines = physicalInfo.split("\n");

    for (let i = 0; i < infoLines.length; i++) {
        const line = infoLines[i];
        const [key, ...valueParts] = line.split("]");
        const value = valueParts.join("]").trim();

        switch (key) {
            case "[Code Name":
                profile.basicInfo.codeName = value;
                break;
            case "[Gender":
                profile.basicInfo.gender = value;
                break;
            case "[Combat Experience":
                profile.basicInfo.combatExperience = value;
                break;
            case "[Place of Birth":
                profile.basicInfo.placeOfBirth = value;
                break;
            case "[Date of Birth":
                profile.basicInfo.dateOfBirth = value;
                break;
            case "[Race":
                profile.basicInfo.race = value;
                break;
            case "[Height":
                profile.basicInfo.height = value;
                break;
            case "[Infection Status":
                // Check if value is empty and the next line exists
                profile.basicInfo.infectionStatus = value || infoLines[i + 1]?.trim() || "";
                break;
        }
    }

    for (let i = 0; i < physicalLines.length; i++) {
        const line = physicalLines[i];
        const [key, ...valueParts] = line.split("]");
        const value = valueParts.join("]").trim();

        switch (key) {
            case "[Physical Strength":
                profile.physicalExam.physicalStrength = value;
                break;
            case "[Mobility":
                profile.physicalExam.mobility = value;
                break;
            case "[Physical Resilience":
                profile.physicalExam.physicalResilience = value;
                break;
            case "[Tactical Acumen":
                profile.physicalExam.tacticalAcumen = value;
                break;
            case "[Combat Skill":
                profile.physicalExam.combatSkill = value;
                break;
            case "[Originium Arts Assimilation":
                profile.physicalExam.originiumArtsAssimilation = value;
                break;
        }
    }

    return profile;
}
