import { YOSTAR_DOMAINS } from "../../../../..";
import { Events } from "../../../../../../../../events";
import emitter from "../../../../../../../../events";
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
        OpenID: email,
        Secret: "",
        Token: emailToken,
        Type: "yostar",
        UserName: email,
    };

    const data = (await (
        await request(YOSTAR_DOMAINS[server] as AKDomain, true, "user/login", {
            method: "POST",
            body: body,
        })
    ).json()) as {
        Code: number;
        Data: {
            AgeVerifyMethod: number;
            IsNew: number;
            UserInfo: {
                ID: string;
                UID2: number;
                PID: string;
                Token: string;
                Birthday: string;
                RegChannel: string;
                TransCode: string;
                State: number;
                DeviceID: string;
                CreatedAt: number;
            };
            Yostar: {
                ID: string;
                Country: string;
                Nickname: string;
                Picture: string;
                State: number;
                AgreeAd: number;
                CreatedAt: number;
            };
        };
        Msg: string;
    };

    if (data.Code !== 200) {
        await emitter.emit(Events.AUTH_YOSTAR_TOKEN_ERROR, {
            email,
            data,
        });
    }

    return {
        uid: data.Data.UserInfo.ID,
        token: data.Data.UserInfo.Token,
    };
};
