import { VOICE_REPOSITORY } from "../..";
import { LangType, type Voice, type Voices } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/voices";
import { STATIC_DATA } from "../../../handler";

export const getAll = (): Voices => {
    const data = STATIC_DATA?.CHARWORD_TABLE as Voices;
    return data;
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
            data: languages.map((language) => ({
                voiceURL: `https://raw.githubusercontent.com/${VOICE_REPOSITORY}/global-server-voices/${language === LangType.CN_MANDARIN ? "voice_cn" : language === LangType.CN_TOPOLECT || language === LangType.GER || language === LangType.ITA || language === LangType.RUS ? "voice_custom" : language === LangType.EN ? "voice_en" : language === LangType.JP ? "voice_jp" : language === LangType.KR ? "voice_kr" : "voice"}/${voice.voiceAsset}.mp3`,
                language,
            })),
            languages,
        }) as Voice;
    }

    return voices;
};
