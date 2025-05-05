import emitter, { Events } from "..";
import colors from "colors";

export const listener = async () => {
    emitter.on(Events.DATABASE_CONNECT, async () => {
        console.log(colors.green("Database connected!"));
    });

    emitter.on(Events.DATABASE_INITIATED, async () => {
        console.log(colors.green("Initiated database!"));
    });

    emitter.on(Events.DATABASE_TABLE_CREATE, async (data) => {
        console.log(colors.gray(`Table created: ${data}`));
    });

    emitter.on(Events.DATABASE_USERS_CREATE, async (data) => {
        console.log(colors.gray(`User created: ${data?.user_id}`));
    });

    emitter.on(Events.DATABASE_USERS_UPDATE, async (data) => {
        console.log(colors.gray(`User updated: ${data?.user_id}`));
    });

    emitter.on(Events.LOCAL_TABLES_DOWNLOADED, async (data) => {
        console.log(colors.gray(`Downloaded table ${data.name}.`));
    });

    emitter.on(Events.LOCAL_TABLES_INITIATED, async () => {
        console.log(colors.green(`Initiated tables!`));
    });

    emitter.on(Events.LOCAL_TABLES_PARSED, async (data) => {
        console.log(colors.gray(`Parsed table ${data.name}.`));
    });

    emitter.on(Events.CONFIG_NETWORK_LOADED, async (data) => {
        console.log(colors.gray(`Loaded network config for ${data}.`));
    });

    emitter.on(Events.CONFIG_VERSION_LOADED, async (data) => {
        console.log(colors.gray(`Loaded version config for ${data}.`));
    });

    emitter.on(Events.CONFIG_NETWORK_INITIATED, async () => {
        console.log(colors.green(`Initiated network config!`));
    });

    emitter.on(Events.CONFIG_VERSION_INITIATED, async () => {
        console.log(colors.green(`Initiated version config!`));
    });

    emitter.on(Events.AUTH_YOSTAR_CODE_SUCCESS, async (data) => {
        console.log(colors.gray(`Code sent to ${data.email} successfully.`));
    });

    emitter.on(Events.AUTH_YOSTAR_CODE_ERROR, async (data) => {
        console.log(colors.red(`Error sending code to ${data.email}:`), data);
    });

    emitter.on(Events.AUTH_YOSTAR_SUBMIT_CODE_ERROR, async (data) => {
        console.log(colors.red(`Error submitting code to ${data.email}:`), data);
    });

    emitter.on(Events.AUTH_YOSTAR_TOKEN_ERROR, async (data) => {
        console.log(colors.red(`Error getting token from ${data.email}:`), data);
    });

    emitter.on(Events.AUTH_YOSTAR_GET_SECRET_ERROR, async (data) => {
        console.log(colors.red(`Error getting secret from ${data.email}:`), data);
    });

    emitter.on(Events.AUTH_YOSTAR_GET_U8_TOKEN_ERROR, async (data) => {
        console.log(colors.red(`Error getting U8 token from ${data.uid}:`), data);
    });

    emitter.on(Events.AUTH_YOSTAR_LOGIN_SUCCESS, async (data) => {
        console.log(colors.green(`Logged in successfully as ${data.uid}.`));
    });

    emitter.on(Events.AUTH_YOSTAR_LOGIN_ERROR, async (data) => {
        console.log(colors.red(`Error logging in as ${data.email}:`), data);
    });

    emitter.on(Events.DPS_CALCULATOR_CLASS_FETCHED, async (data) => {
        console.log(colors.gray(`Fetched class ${data.name}.`));
    });

    emitter.on(Events.DPS_ENEMY_CLASS_FETCHED, async (data) => {
        console.log(colors.gray(`Fetched enemy class ${data.name}.`));
    });

    emitter.on(Events.DPS_CALCULATOR_INITIATED, async () => {
        console.log(colors.green(`Initiated DPS calculator!`));
    });
};
