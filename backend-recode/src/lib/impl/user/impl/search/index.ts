import type { AKServer, AuthSession } from "../../../../../types/impl/lib/impl/authentication";
import { SearchResults } from "../../../../../types/impl/lib/impl/user/impl/search";
import { authRequest } from "../../../authentication/impl/request/impl/auth";

export const search = async (session: AuthSession, server: AKServer, nickName: string, nickNumber?: string, limit?: number) => {
    if (nickName.includes("#")) nickName = nickName.split("#")[0];

    const sortKey = ["level"];

    const req = (await (
        await authRequest(
            "social/getSortListInfo",
            session,
            {
                body: JSON.stringify({
                    type: 0,
                    sortKeyList: sortKey,
                    param: {
                        nickName,
                        nickNumber: nickNumber || "",
                    },
                }),
            },
            server,
        )
    ).json()) as {
        result: {
            uid: string;
            level: number;
            infoShare: number;
        }[];
    };

    const uidData = req.result.sort((a, b) => {
        const aValue = sortKey.map((key) => a[key as keyof typeof a]);
        const bValue = sortKey.map((key) => b[key as keyof typeof b]);
        for (let i = 0; i < aValue.length; i++) {
            if (aValue[i] < bValue[i]) return 1;
            if (aValue[i] > bValue[i]) return -1;
        }
        return 0;
    });

    const uidList = uidData.slice(0, limit ? limit : uidData.length).map((item: { uid: string }) => item.uid);

    const data = (await (await authRequest("social/searchPlayer", session, { body: JSON.stringify({ idList: uidList }) }, server)).json()) as SearchResults;
    return data.players ?? [];
};
