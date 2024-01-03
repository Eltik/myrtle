import { v4 as uuidv4 } from "uuid";
import random from "random";
import * as crypto from "crypto";
import * as querystring from "querystring";
import * as readline from "readline";

const PASSPORT_DOMAINS = {
    en: "https://passport.arknights.global",
    jp: "https://passport.arknights.jp",
    kr: "https://passport.arknights.kr",
    cn: "ERROR",
    bili: "ERROR",
    tw: "ERROR"
};
const NETWORK_ROUTES = {
    en: "https://ak-conf.arknights.global/config/prod/official/network_config",
    jp: "https://ak-conf.arknights.jp/config/prod/official/network_config",
    kr: "https://ak-conf.arknights.kr/config/prod/official/network_config",
    cn: "https://ak-conf.hypergryph.com/config/prod/official/network_config",
    bili: "https://ak-conf.hypergryph.com/config/prod/b/network_config",
    tw: "https://ak-conf.txwy.tw/config/prod/official/network_config",
};

const DEFAULT_HEADERS = {
    "Content-Type": "application/json",
    "X-Unity-Version": "2017.4.39f1",
    "User-Agent": "Dalvik/2.1.0 (Linux; U; Android 11; KB2000 Build/RP1A.201005.001)",
    Connection: "Keep-Alive"
};

type AKDistributor = "yostar" | "hypergryph" | "bilibili" | "longcheng";
type AKServer = "en" | "jp" | "kr" | "cn" | "bili" | "tw";
type AKDomain = "gs" | "as" | "u8" | "hu" | "hv" | "rc" | "an" | "prean" | "sl" | "of" | "pkgAd" | "pkgIOS";

export class AuthSession {
    public uid: string;
    public secret: string;
    public seqnum: number = 1;

    constructor(uid?: string, secret?: string, seqnum?: number) {
        this.uid = uid || "";
        this.secret = secret || "";
        this.seqnum = seqnum || 1;
    }
}

export class Auth {
    public server: AKServer;
    public device_ids: [string, string, string];
    public session: AuthSession = new AuthSession();

    private domains: { [key in AKServer]: { [key in AKDomain]: string } };
    private versions: Record<AKServer, Record<"resVersion" | "clientVersion", string>>;

    /**
     * @param {string|null} server - Arknights server.
     * @param {Object|null} network - Network session.
     */
    constructor(server: AKServer | null = null) {
        if (server === null) {
            server = "en";
        }

        this.server = server || "en";
        this.device_ids = this.createRandomDeviceIds();
        this.domains = {} as { [key in AKServer]: { [key in AKDomain]: string } };
        this.versions = {} as Record<AKServer, Record<"resVersion" | "clientVersion", string>>;
        for (let server in NETWORK_ROUTES) {
            Object.assign(this.domains, {
                [server as AKServer]: {}
            });
            Object.assign(this.versions, {
                [server as AKServer]: {
                    resVersion: "",
                    clientVersion: ""
                }
            })
        }
    }

    public async getSecret(uid: string, u8Token: string): Promise<string> {
        console.log(`Getting session secret for ${uid}...`);

        if (!this.versions[this.server]["resVersion"]) {
            await this.loadVersionConfig();
        }

        const networkVersion = {
            cn: "5",
            bili: "5",
            en: "1",
            jp: "1",
            kr: "1",
            "tw": "ERROR"
        }[this.server];

        const body = {
            platform: 1,
            networkVersion: networkVersion,
            assetsVersion: this.versions[this.server]["resVersion"],
            clientVersion: this.versions[this.server]["clientVersion"],
            token: u8Token,
            uid: uid,
            deviceId: this.device_ids[0],
            deviceId2: this.device_ids[1],
            deviceId3: this.device_ids[2],
        };

        const headers = {
            secret: "",
            seqnum: "1",
            uid: uid,
        };

        const data = await (await this.request("gs", "account/login", {
            body: JSON.stringify(body),
            headers
        })).json();

        const secret = data.secret;

        this.session.secret = secret;
        console.log(`Logged in with UID ${uid}.`)
        return secret;
    }

    public async getU8Token(channelUID: string, accessToken: string): Promise<[string, string]> {
        console.log(`Getting U8 token for ${channelUID}...`);
        const channelID = {
            cn: "1",
            bili: "2",
            en: "3",
            jp: "3",
            kr: "3",
            tw: "ERROR"
        }[this.server];

        let extension: { uid: string, token: string } | { uid: string, access_token: string } | null = null;
        if (channelID === "3") {
            extension = {
                uid: channelUID,
                token: accessToken
            }
        } else {
            extension = {
                uid: channelUID,
                access_token: accessToken
            }
        }

        const body = {
            appId: "1",
            platform: 1,
            channelId: channelID,
            subChannel: channelID,
            extension: JSON.stringify(extension),
            
            // Optional fields
            worldId: channelID,
            deviceId: this.device_ids[0],
            deviceId2: this.device_ids[1],
            deviceId3: this.device_ids[2],
        }

        /*
        Object.assign(body, {
            sign: this.generateU8Sign(body)
        })
        */

        const data = await (await this.request("u8", "user/v1/getToken", {
            body: JSON.stringify(body)
        })).json();
        console.log(data);
        
        const uid = data["uid"];
        const token = data["token"];

        this.session.uid = uid
        return [uid, token];
    }

    public async fromToken(server: AKServer, channelUID: string, token: string): Promise<Auth | null> {
        let auth: Auth | null = null;
        if (["en", "jp", "kr"].includes(server)) {
            auth = new YostarAuth(server);
            await auth.loginWithToken(channelUID, token);
        } else if (server === "cn") {
            // Hypergryph
        } else if (server === "bili") {
            // Bilbili
        } else if (server === "tw") {
            // Longcheng
        } else {
            throw new Error(`Invalid server ${server}.`);
        }
        return auth;
    }

    public async loginWithToken(channelUID: string, token: string) {
        throw new Error("Unimplemented");
    }

    public async authRequest(endpoint: string, args?: RequestInit, server?: AKServer): Promise<Response> {
        server = server ? server : "en";
        if (!this.session.uid) {
            throw new Error("Not logged in.");
        }

        return await this.request("gs", endpoint, args, server);
    }

    public async request(domain: AKDomain, endpoint: string | null = null, args?: RequestInit, server?: AKServer): Promise<Response> {
        server = server ? server : "en";

        let url: string = "";
        if (domain.includes("http")) {
            url = domain;
        } else {
            if (server === null || server === undefined) {
                throw new Error("No default server set.");
            }
            if (!this.domains.hasOwnProperty(server)) {
                throw new Error(`Invalid server for server ${server}.`);
            }
            if (!this.domains[server] || domain === "hv") {
                await this.loadNetworkConfig(server);
            }
            if (!this.domains[server].hasOwnProperty(domain)) {
                throw new Error(`Invalid domain for domain ${domain}.`);
            }

            url = this.domains[server][domain]
        }

        if (url.includes("{0}")) {
            url = url.replace("{0}", "Android");
        }
        if (endpoint) {
            url = url + "/" + endpoint;
        }

        if (!args) {
            args = {};
        }
        if (!args.method) {
            args.method = args.body ? "POST" : "GET";
        }
        if (!args.headers) {
            args.headers = DEFAULT_HEADERS;
        } else {
            Object.assign(args.headers, DEFAULT_HEADERS)
        }

        const data = await (await fetch(url, args));
        return data;
    }

    public async loadNetworkConfig(server: AKServer | "all" | null = null): Promise<void> {
        if (server === null) {
            server = this.server;
        }

        if (server === "all") {
            for (const server of ["en", "jp", "kr", "cn", "bili", "tw"]) {
                await this.loadNetworkConfig(server as AKServer);
            }
            return;
        }

        console.log(`Loading network config for ${server}...`);
        const request = await (await this.request(NETWORK_ROUTES[server] as AKDomain));
        const data = JSON.parse((await request.json())["content"]);
        Object.assign(this.domains[server], data["configs"][data["funcVer"]]["network"]);
    }

    public async loadVersionConfig(server: AKServer | "all" | null = null): Promise<void> {
        if (server === null) {
            server = this.server;
        }

        if (server === "all") {
            for (const server of ["en", "jp", "kr", "cn", "bili", "tw"]) {
                await this.loadVersionConfig(server as AKServer);
            }
            return;
        }

        console.log(`Loading version config for ${server}...`);
        const request = await (await this.request("hv", null, undefined, server));
        const data = await request.json();
        Object.assign(this.versions[server], data);
    }

    private createRandomDeviceIds(): [string, string, string] {
        // TODO: Fix
        const deviceid2 = "86" + Array.from({ length: 13 }, () => random.int(0, 9)).join('');
        return [uuidv4(), deviceid2, uuidv4()];
    }

    private generateU8Sign(data: {
        appId: string,
        platform: number,
        channelId: string,
        subChannel: string,
        extension: string,
        
        // Optional fields
        worldId: string,
        deviceId: string,
        deviceId2: string,
        deviceId3: string
    }): string {
        const query = querystring.stringify(Object.entries(data).sort((a, b) => a[0].localeCompare(b[0])) as any);

        const hamaCode = crypto.createHmac('sha1', '91240f70c09a08a6bc72af1a5c8d4670')
            .update(query)
            .digest('hex')
            .toLowerCase();

        return hamaCode;
    }
}

export class YostarAuth extends Auth {
    constructor(server: AKServer | null = null) {
        super(server);
    }

    public async requestPassport(endpoint: string, args?: RequestInit): Promise<Response> {
        return await this.request(PASSPORT_DOMAINS[this.server] as AKDomain, endpoint, args);
    }

    public async getAccessToken(channelUID: string, yostarToken: string): Promise<string> {
        const body = {
            platform: "android",
            uid: channelUID,
            token: yostarToken,
            deviceId: this.device_ids[0],
        }
        const data = await (await this.requestPassport("user/login", {
            body: JSON.stringify(body)
        })).json();
        return data.accessToken;
    }

    public async requestYostarAuth(email: string) {
        console.log(`Sending code to ${email}...`);
        const body = {
            platform: "android",
            account: email,
            authlang: "en"
        };
        const response = await this.requestPassport("account/yostar_auth_request", {
            body: JSON.stringify(body)
        });

        return await response.json();
    }

    public async submitYostarAuth(email: string, code: string): Promise<[string, string]> {
        const body = {
            account: email,
            code: code
        };

        const data = await (await this.requestPassport("account/yostar_auth_submit", {
            body: JSON.stringify(body)
        })).json();
        return [data.yostar_uid, data.yostar_token];
    }

    public async getYostarToken(email: string, yostarUID: string, yostarToken: string): Promise<[string, string]> {
        const body = {
            yostar_username: email,
            yostar_uid: yostarUID,
            yostar_token: yostarToken,
            deviceId: this.device_ids[0],
            createNew: "0",
        };
        const data = await (await this.requestPassport("user/yostar_createlogin", {
            body: JSON.stringify(body)
        })).json();
        return [data.uid, data.token];
    }

    public async createGuestAccount(): Promise<[string, string]> {
        const body = {
            deviceId: this.device_ids[0],
        }
        const data = await (await this.requestPassport("user/create", {
            body: JSON.stringify(body)
        })).json();
        console.log(`Created guest account ${data.uid}`)
        return [data.uid, data.token];
    }

    public async bindNickname(nickname: string) {
        /*
        await this.authRequest("user/bindNickname", {
            body: JSON.stringify({
                nickName: nickname
            })
        });
        */
    }

    override async loginWithToken(channelUID: string, yostarToken: string) {
        const accessToken = await this.getAccessToken(channelUID, yostarToken);
        const data = await this.getU8Token(channelUID, accessToken);

        this.session.uid = data[0];
        const u8Token = data[1];
        await this.getSecret(this.session.uid, u8Token);
    }

    public async getTokenFromEmailCode(email: string | undefined = undefined, code: string | undefined = undefined, stdin: boolean = false): Promise<[string, string]> {
        if (!email) {
            if (!stdin) {
                throw new Error("Email not provided but stdin is disabled.");
            }

            email = await this.getUserInput("Enter email:");
        }

        if (!code) {
            await this.requestYostarAuth(email);
            if (!stdin) {
                return ["", ""];
            }

            console.log(`Code sent to ${email}.`); // noqa: T201
            code = await this.getUserInput("Enter code: ");
        }

        const [yostarUid, yostarToken] = await this.submitYostarAuth(email, code);
        console.log(`Yostar UID: ${yostarUid} Yostar Token: ${yostarToken}`);
        return this.getYostarToken(email, yostarUid, yostarToken);
    }

    private async getUserInput(prompt: string): Promise<string> {
        const rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout
        });

        return new Promise<string>((resolve) => {
            rl.question(prompt, (answer) => {
                rl.close();
                resolve(answer);
            });
        });
    }

    public async loginWithEmailCode(email?: string, code?: string, stdin: boolean = false) {
        const data = await this.getTokenFromEmailCode(email, code, stdin);
        
        const channelUID = data[0];
        const token = data[1];
        await this.loginWithToken(channelUID, token);

        if (stdin) {
            console.log(`Channel UID: ${channelUID} Token: ${token}`);
            console.log(`Usage: loginWithToken("${channelUID}", "${token}")`)
        }

        return [channelUID, token];
    }

    public async loginAsGuest(nickname?: string): Promise<[string, string]> {
        const data = await this.createGuestAccount();
        
        const channelUID = data[0];
        const token = data[1];

        await this.loginWithToken(channelUID, token);
        await this.bindNickname(nickname ?? "Guest");
        return [channelUID, token];
    }
}