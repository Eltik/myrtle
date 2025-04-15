import { YOSTAR_DOMAINS } from "../../../../..";
import type { AKDomain, AKServer } from "../../../../../../../../types/impl/lib/impl/authentication";
import request from "../../../../request";

export const requestToken = async (
    email: string,
    emailToken: string,
    server: AKServer,
): Promise<{
    uid: string;
    token: string;
}> => {
    const body = {
        CheckAccount: 0,
        Geetest: {
            CaptchaId: null,
            CaptchaOutput: null,
            GenTime: null,
            LotNumber: null,
            PassToken: null,
        },
        OpenId: email,
        Secret: "",
        Token: emailToken,
        Type: "yostar",
        UserName: email,
    };

    const data = (await (
        await request(YOSTAR_DOMAINS[server] as AKDomain, "user/login", {
            body: JSON.stringify(body),
        })
    ).json()) as {
        Data: {
            UserInfo: {
                ID: string;
                Token: string;
            };
        };
    };

    return {
        uid: data.Data.UserInfo.ID,
        token: data.Data.UserInfo.Token,
    };
};
