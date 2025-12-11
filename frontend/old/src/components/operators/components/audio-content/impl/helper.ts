import { LangType, PlaceType } from "~/types/impl/api/static/voices";

// Helper function to get language name from LangType
export const getLangTypeName = (langType: LangType): string => {
    switch (langType) {
        case LangType.JP:
            return "Japanese";
        case LangType.CN_MANDARIN:
            return "Chinese";
        case LangType.CN_TOPOLECT:
            return "Chinese (Regional)";
        case LangType.KR:
            return "Korean";
        case LangType.EN:
            return "English";
        case LangType.RUS:
            return "Russian";
        case LangType.ITA:
            return "Italian";
        case LangType.GER:
            return "German";
        default:
            return "Unknown";
    }
};

// Helper function to convert PlaceType to user-friendly category name
export const getCategoryName = (placeType: PlaceType): string => {
    switch (placeType) {
        case PlaceType.GREETING:
            return "Greetings";
        case PlaceType.BATTLE_START:
        case PlaceType.BATTLE_FACE_ENEMY:
        case PlaceType.BATTLE_SELECT:
        case PlaceType.BATTLE_PLACE:
        case PlaceType.BATTLE_SKILL_1:
        case PlaceType.BATTLE_SKILL_2:
        case PlaceType.BATTLE_SKILL_3:
        case PlaceType.BATTLE_SKILL_4:
            return "Combat";
        case PlaceType.HOME_PLACE:
        case PlaceType.HOME_SHOW:
        case PlaceType.HOME_WAIT:
            return "Interaction";
        case PlaceType.BUILDING_PLACE:
        case PlaceType.BUILDING_TOUCHING:
        case PlaceType.BUILDING_FAVOR_BUBBLE:
            return "Base";
        case PlaceType.LEVEL_UP:
            return "Level Up";
        case PlaceType.GACHA:
            return "Recruitment";
        case PlaceType.SQUAD:
        case PlaceType.SQUAD_FIRST:
            return "Squad";
        default:
            return "Other";
    }
};

// Helper to get description for voice line
export const getVoiceDescription = (placeType: PlaceType): string => {
    switch (placeType) {
        case PlaceType.GREETING:
            return "Greeting";
        case PlaceType.BATTLE_START:
            return "Battle Start";
        case PlaceType.BATTLE_FACE_ENEMY:
            return "Facing Enemy";
        case PlaceType.BATTLE_SELECT:
            return "Selected in Battle";
        case PlaceType.BATTLE_PLACE:
            return "Deployment";
        case PlaceType.BATTLE_SKILL_1:
            return "Activating Skill 1";
        case PlaceType.BATTLE_SKILL_2:
            return "Activating Skill 2";
        case PlaceType.BATTLE_SKILL_3:
            return "Activating Skill 3";
        case PlaceType.HOME_PLACE:
            return "Assigned as Assistant";
        case PlaceType.HOME_SHOW:
            return "Conversation 1";
        case PlaceType.HOME_WAIT:
            return "Conversation 2";
        case PlaceType.BUILDING_PLACE:
            return "Assigned to Facility";
        case PlaceType.BUILDING_TOUCHING:
            return "Interacting in Base";
        case PlaceType.BUILDING_FAVOR_BUBBLE:
            return "Trust Tap";
        case PlaceType.LEVEL_UP:
            return "Level Up";
        case PlaceType.GACHA:
            return "Recruitment";
        case PlaceType.SQUAD:
            return "Added to Squad";
        case PlaceType.SQUAD_FIRST:
            return "First Time in Squad";
        default:
            return placeType.toString();
    }
};
