import { Database } from "bun:sqlite";

export const db = new Database("db.sqlite");
export const tableName: string = "players";

export const init = async () => {
    const players = `
        CREATE TABLE IF NOT EXISTS ${tableName} (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uid TEXT NOT NULL,
            server TEXT NOT NULL,
            data JSON NOT NULL
        )
    `;

    db.query(players).run();
};
