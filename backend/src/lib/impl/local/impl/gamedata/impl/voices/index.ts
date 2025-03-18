import { VOICE_REPOSITORY } from "../..";
import { LangType, type Voice, type Voices } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/voices";
import { STATIC_DATA } from "../../../handler";

export const getAll = (): Voices => {
    const data = STATIC_DATA?.CHARWORD_TABLE as Voices;
    return data;
};

const isCustom = (lang: LangType) => {
    return lang !== LangType.EN && lang !== LangType.JP && lang !== LangType.KR && lang !== LangType.CN_MANDARIN;
};

const appendedCustom = (lang: LangType) => {
    switch (lang) {
        case LangType.GER:
            return "_ger";
        case LangType.ITA:
            return "_ita";
        case LangType.RUS:
            return "_rus";
        case LangType.CN_TOPOLECT:
            return "_cn_topolect";
        default:
            return "";
    }
};

export default (id: string): Voice[] | null => {
    const charWordData = getAll();
    const voices = Object.entries(charWordData.charWords)
        .map(([id, voice]) => ({
            id,
            ...voice,
        }))
        .filter((item) => item.charId === id);

    const voiceLangDict = Object.entries(charWordData.voiceLangDict).map(([id, voiceLang]) => ({
        id,
        ...voiceLang,
    }));

    for (const voice of voices) {
        const voiceLang = voiceLangDict.find((item) => item.charId === voice.charId);
        if (!voiceLang) continue;

        const languages = Object.values(voiceLang.dict).map((item) => item.voiceLangType);

        Object.assign(voice, {
            data: languages.map((language) => {
                const voiceAsset = voice.voiceAsset;
                let dir = voiceAsset.split("/")[0];
                const file = voiceAsset.split("/")[1];

                if (!dir.includes(LangType.CN_TOPOLECT)) {
                    if (!isCustom(language) && voiceAsset.split("/")[0].split("_").length > 3) {
                        dir = voiceAsset.split("/")[0].split("_").slice(0, -1).join("_");
                    } else if (isCustom(language) && voiceAsset.split("/")[0].split("_").length > 3) {
                        dir = voiceAsset.split("/")[0];
                    } else if (isCustom(language)) {
                        const appended = appendedCustom(language);
                        dir = voiceAsset.split("/")[0] + appended;
                    }
                } else {
                    if (!isCustom(language) && voiceAsset.split("/")[0].split("_").length > 4) {
                        dir = voiceAsset.split("/")[0].split("_").slice(0, -2).join("_");
                    } else if (isCustom(language) && voiceAsset.split("/")[0].split("_").length > 4) {
                        dir = voiceAsset.split("/")[0];
                    } else if (isCustom(language)) {
                        const appended = appendedCustom(language);
                        dir = voiceAsset.split("/")[0] + appended;
                    }
                }

                return {
                    voiceURL: `https://raw.githubusercontent.com/${VOICE_REPOSITORY}/global-server-voices/${language === LangType.CN_MANDARIN ? "voice_cn" : language === LangType.CN_TOPOLECT || language === LangType.GER || language === LangType.ITA || language === LangType.RUS ? "voice_custom" : language === LangType.EN ? "voice_en" : language === LangType.JP ? "voice" : language === LangType.KR ? "voice_kr" : "voice"}/${dir.toLowerCase()}/${file}.mp3`,
                    language,
                    cvName: voiceLang.dict[language].cvName,
                };
            }),
            languages,
        }) as Voice;
    }

    return voices;
};
