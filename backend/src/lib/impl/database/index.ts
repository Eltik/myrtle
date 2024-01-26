import { Database } from "bun:sqlite";

export const db = new Database("db.sqlite");
export const tableName: string = "players";

export const init = async () => {
    const players = `
        CREATE TABLE IF NOT EXISTS ${tableName} (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uid TEXT NOT NULL,
            server TEXT NOT NULL,
            dungeon JSON NOT NULL,
            activity JSON NOT NULL,
            status JSON NOT NULL,
            troop JSON NOT NULL,
            "npcAudio" JSON NOT NULL,
            "pushFlags" JSON NOT NULL,
            equipment JSON NOT NULL,
            skin JSON NOT NULL,
            shop JSON NOT NULL,
            mission JSON NOT NULL,
            social JSON NOT NULL,
            building JSON NOT NULL,
            "dexNav" JSON NOT NULL,
            crisis JSON NOT NULL,
            tshop JSON NOT NULL,
            gacha JSON NOT NULL,
            backflow JSON NOT NULL,
            mainline JSON NOT NULL,
            avatar JSON NOT NULL,
            background JSON NOT NULL,
            rlv2 JSON NOT NULL,
            deepSea JSON NOT NULL,
            tower JSON NOT NULL,
            "siracusaMap" JSON NOT NULL,
            storyreview JSON NOT NULL,
            medal JSON NOT NULL,
            "aprilFool" JSON NOT NULL,
            retro JSON NOT NULL,
            charm JSON NOT NULL,
            carousel JSON NOT NULL,
            consumable JSON NOT NULL,
            event JSON NOT NULL,
            "collectionReward" JSON NOT NULL,
            recruit JSON NOT NULL,
            "checkIn" JSON NOT NULL,
            car JSON NOT NULL,
            "openServer" JSON NOT NULL,
            "campaignsV2" JSON NOT NULL,
            inventory JSON NOT NULL,
            "limitedBuff" JSON NOT NULL,
            ticket JSON NOT NULL
        )
    `;

    db.query(players).run();
};
