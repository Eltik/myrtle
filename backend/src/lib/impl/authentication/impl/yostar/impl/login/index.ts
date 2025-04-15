import emitter, { Events } from "../../../../../../../events";
import { AuthSession, type AKServer } from "../../../../../../../types/impl/lib/impl/authentication";
import { getSecret } from "./impl/getSecret";
import { getU8Token } from "./impl/getU8Token";
import { requestToken } from "./impl/requestToken";
import { submitYostarAuth } from "./impl/submitYostar";

export default async (email: string, code: string, server: AKServer, session?: AuthSession): Promise<AuthSession> => {
    const yostarData = await submitYostarAuth(email, code, server);
    const tokenData = await requestToken(email, yostarData.token, server);

    session = session ? session : new AuthSession();

    const data = await getU8Token(tokenData.uid, tokenData.token, server);

    Object.assign(session, {
        uid: data.uid,
    });

    const u8Token = data.token;
    const secret = await getSecret(data.uid, u8Token, server);

    Object.assign(session, {
        token: u8Token,
        secret: secret,
    });

    await emitter.emit(Events.AUTH_YOSTAR_LOGIN_SUCCESS, session);

    return session;
};
