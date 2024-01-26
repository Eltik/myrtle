import { db, tableName } from "..";

type SQLColumn = {
    cid: number;
    name: string;
    type: string;
    notnull: number;
    dflt_value: string;
    pk: number;
};

export const schema = async () => {
    const existingColumnsQuery = `PRAGMA table_info("${tableName}");`;
    const existingColumns = db.query(existingColumnsQuery).all() as SQLColumn[];
    return existingColumns;
};
