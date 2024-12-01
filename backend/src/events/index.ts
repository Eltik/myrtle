import EventEmitter2 from "eventemitter2";

export enum Events {
    DATABASE_CONNECT = "database.connect",
    DATABASE_INITIATED = "database.initiated",
    DATABSE_TABLES_INITIATED = "database.tables.initiated",
    DATABASE_TABLE_CREATE = "database.table.create",

    DATABASE_USERS_CREATE = "database.users.create",
    DATABASE_USERS_UPDATE = "database.users.update",

    LOCAL_TABLES_DOWNLOADED = "local.tables.downloaded",
    LOCAL_TABLES_PARSED = "local.tables.parsed",
    LOCAL_TABLES_INITIATED = "local.tables.initiated",

    CONFIG_NETWORK_LOADED = "config.network.loaded",
    CONFIG_VERSION_LOADED = "config.version.loaded",

    CONFIG_NETWORK_INITIATED = "config.network.initiated",
    CONFIG_VERSION_INITIATED = "config.version.initiated",

    AUTH_YOSTAR_CODE_SUCCESS = "auth.yostar.code.success",
    AUTH_YOSTAR_CODE_ERROR = "auth.yostar.code.error",
    AUTH_YOSTAR_LOGIN_SUCCESS = "auth.yostar.login.success",
    AUTH_YOSTAR_LOGIN_ERROR = "auth.yostar.login.error",
}

const emitter = new EventEmitter2({});

export default emitter;
