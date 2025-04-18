import emitter, { Events } from "../../../../../events";
import { ExcelTables } from "../../../../../types/impl/lib/impl/local/impl/handler";
import { download } from "./impl/download";
import { exists } from "./impl/exists";
import { get } from "./impl/get";
import { join } from "path";
import { init as initChibis } from "../gamedata/impl/chibi";

export const GAME_DATA_REPOSITORY = "Kengxxiao/ArknightsGameData_YoStar";

export let STATIC_DATA: Record<keyof typeof ExcelTables, any> | null = null;

const dataObject: Record<keyof typeof ExcelTables, any> = {
    ACTIVITY_TABLE: undefined,
    AUDIO_DATA: undefined,
    BATTLE_EQUIP_TABLE: undefined,
    BUILDING_DATA: undefined,
    CAMPAIGN_TABLE: undefined,
    CHAPTER_TABLE: undefined,
    CHAR_META_TABLE: undefined,
    CHAR_PATCH_TABLE: undefined,
    CHARACTER_TABLE: undefined,
    CHARM_TABLE: undefined,
    CHARWORD_TABLE: undefined,
    CHECKIN_TABLE: undefined,
    CLIMB_TOWER_TABLE: undefined,
    CLUE_DATA: undefined,
    CRISIS_TABLE: undefined,
    CRISIS_V2_TABLE: undefined,
    DATA_VERSION: undefined,
    DISPLAY_META_TABLE: undefined,
    ENEMY_HANDBOOK_TABLE: undefined,
    FAVOR_TABLE: undefined,
    GACHA_TABLE: undefined,
    GAMEDATA_CONST: undefined,
    HANDBOOK_INFO_TABLE: undefined,
    HANDBOOK_TABLE: undefined,
    HANDBOOK_TEAM_TABLE: undefined,
    ITEM_TABLE: undefined,
    MEDAL_TABLE: undefined,
    MISSION_TABLE: undefined,
    OPEN_SERVER_TABLE: undefined,
    PLAYER_AVATAR_TABLE: undefined,
    RANGE_TABLE: undefined,
    REPLICATE_TABLE: undefined,
    RETRO_TABLE: undefined,
    ROGUELIKE_TABLE: undefined,
    ROGUELIKE_TOPIC_TABLE: undefined,
    SANDBOX_PERM_TABLE: undefined,
    SANDBOX_TABLE: undefined,
    SHOP_CLIENT_TABLE: undefined,
    SKILL_TABLE: undefined,
    SKIN_TABLE: undefined,
    STAGE_TABLE: undefined,
    STORY_REVIEW_META_TABLE: undefined,
    STORY_REVIEW_TABLE: undefined,
    STORY_TABLE: undefined,
    TECH_BUFF_TABLE: undefined,
    TIP_TABLE: undefined,
    TOKEN_TABLE: undefined,
    UNIEQUIP_DATA: undefined,
    UNIEQUIP_TABLE: undefined,
    ZONE_TABLE: undefined,
};

export const init = async () => {
    const promises: Promise<void>[] = [];

    // Edge case with chibis
    await initChibis();

    const values = Object.values(ExcelTables);

    for (const value of values) {
        promises.push(
            new Promise(async (resolve) => {
                const downloaded = await exists(value as ExcelTables);
                if (!downloaded) {
                    try {
                        await download(value as ExcelTables);
                        await emitter.emit(Events.LOCAL_TABLES_DOWNLOADED, {
                            name: value,
                        });
                    } catch {
                        console.error(`Failed to download ${value} from ${GAME_DATA_REPOSITORY}`);
                    }
                }
                resolve();
            }),
        );
    }

    await Promise.all(promises);

    await updateStaticData();
};

export const updateStaticData = async () => {
    const keys = Object.keys(ExcelTables);

    const staticPromises = keys.map(async (key) => {
        const value = ExcelTables[key as keyof typeof ExcelTables] as ExcelTables;
        const data = await get(value);
        if (data) {
            dataObject[key as keyof typeof ExcelTables] = data;

            await emitter.emit(Events.LOCAL_TABLES_PARSED, {
                name: key,
            });
        } else {
            console.error(`Failed to parse ${value} from ${join(import.meta.dir, `./data/${value}.json`)}`);
        }
    });

    await Promise.all(staticPromises);

    STATIC_DATA = dataObject;

    await emitter.emit(Events.LOCAL_TABLES_INITIATED);
};
