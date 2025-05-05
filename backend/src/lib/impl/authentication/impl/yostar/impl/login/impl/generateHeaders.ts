import type { AKServer } from "../../../../../../../../types/impl/lib/impl/authentication";
import { createHash, randomUUID } from "crypto";
import { DEFAULT_HEADERS } from "../../../../..";

export const generateHeaders = (body: string, server: AKServer = "en", yostarUID: string | null = null, yostarToken: string | null = null, deviceId: string | null = null) => {
    const headers = {
        PID: server === "en" ? "US-ARKNIGHTS" : server === "jp" ? "JP-AK" : "KR-ARKNIGHTS",
        Channel: "googleplay",
        Platform: "android",
        Version: "4.10.0",
        GVersionNo: "2000112",
        GBuildNo: "",
        Lang: server === "en" ? "en" : server === "jp" ? "jp" : "ko",
        DeviceID: deviceId || randomUUID(),
        DeviceModel: "F9",
        UID: yostarUID || "",
        Token: yostarToken || "",
        Time: Math.floor(Date.now() / 1000),
    };

    const jsonString = JSON.stringify(headers, null, "");

    const md5Hash = createHash("md5")
        .update(jsonString + body + "886c085e4a8d30a703367b120dd8353948405ec2")
        .digest("hex");

    const headerAuth = {
        Head: headers,
        Sign: md5Hash.toUpperCase(),
    };

    return {
        "X-Unity-Version": DEFAULT_HEADERS["X-Unity-Version"],
        "User-Agent": DEFAULT_HEADERS["User-Agent"],
        Connection: DEFAULT_HEADERS["Connection"],
        Authorization: JSON.stringify(headerAuth, null, ""),
        "Content-Type": "application/json",
    };
};
