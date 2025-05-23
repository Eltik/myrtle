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
    AUTH_YOSTAR_SUBMIT_CODE_ERROR = "auth.yostar.submit.code.error",
    AUTH_YOSTAR_TOKEN_ERROR = "auth.yostar.token.error",
    AUTH_YOSTAR_GET_SECRET_ERROR = "auth.yostar.get.secret.error",
    AUTH_YOSTAR_GET_U8_TOKEN_ERROR = "auth.yostar.get.u8.token.error",
    AUTH_YOSTAR_CODE_ERROR = "auth.yostar.code.error",
    AUTH_YOSTAR_LOGIN_SUCCESS = "auth.yostar.login.success",
    AUTH_YOSTAR_LOGIN_ERROR = "auth.yostar.login.error",

    DPS_CALCULATOR_CLASS_FETCHED = "dps.calculator.class.fetched",
    DPS_ENEMY_CLASS_FETCHED = "dps.enemy.class.fetched",
    DPS_CALCULATOR_INITIATED = "dps.calculator.initiated",
}

const emitter = new EventEmitter2({});

export default emitter;
