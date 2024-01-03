import type { Auth } from "./auth";

export class Client {
    private auth: Auth;

    constructor(auth: Auth) {
        this.auth = auth;
    }

    public async initAuth() {
        await this.auth.loadNetworkConfig();
        await this.auth.loadVersionConfig();
    }

    public async getRawData() {
        const data = await (await this.auth.authRequest("account/syncData", {
            body: JSON.stringify({
                platform: 1
            })
        })).json();
        return data;
    }

    public async getSocialSortList(type: number, sortKey = ["level"], param = {}, { server = null } = {}) {
        const data = await (await this.auth.authRequest("social/getSocialSortList", {
            body: JSON.stringify({
                type,
                sortKey,
                param,
                server
            })
        })).json();
        return data;
    }
}