import type { AKDomain, AKServer } from "../../../types/impl/lib/impl/authentication";
import load from "./impl/load";

export const YOSTAR_DOMAINS = {
    en: "https://en-sdk-api.yostarplat.com",
    jp: "https://jp-sdk-api.yostarplat.com",
    kr: "https://jp-sdk-api.yostarplat.com",
    cn: "ERROR",
    bili: "ERROR",
    tw: "ERROR",
};

export const NETWORK_ROUTES = {
    en: "https://ak-conf.arknights.global/config/prod/official/network_config",
    jp: "https://ak-conf.arknights.jp/config/prod/official/network_config",
    kr: "https://ak-conf.arknights.kr/config/prod/official/network_config",
    cn: "https://ak-conf.hypergryph.com/config/prod/official/network_config",
    bili: "https://ak-conf.hypergryph.com/config/prod/b/network_config",
    tw: "https://ak-conf-tw.gryphline.com/config/prod/official/network_config",
};

export const DEFAULT_HEADERS = {
    "Content-Type": "application/json",
    "X-Unity-Version": "2017.4.39f1",
    "User-Agent": "Dalvik/2.1.0 (Linux; U; Android 11; KB2000 Build/RP1A.201005.001)",
    Connection: "Keep-Alive",
};

export const DEVICE_IDS: [string, string, string] = ["", "", ""];
export const DOMAINS: { [key in AKServer]: { [key in AKDomain]: string } } = {} as { [key in AKServer]: { [key in AKDomain]: string } };
export const VERSIONS: Record<AKServer, Record<"resVersion" | "clientVersion", string>> = {} as Record<AKServer, Record<"resVersion" | "clientVersion", string>>;

export const init = async () => {
    await load();
};
