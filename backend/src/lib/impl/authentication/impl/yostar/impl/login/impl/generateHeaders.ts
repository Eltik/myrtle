import type { AKServer } from "../../../../../../../../types/impl/lib/impl/authentication";
import { v4 as uuidv4 } from "uuid";
import { createHash } from "crypto";

export const generateHeaders = (body: string, server: AKServer = "en", yostarUID: string | null = null, yostarToken: string | null = null, deviceId: string | null = null) => {
    // If deviceId is provided, use it; otherwise generate one
    const actualDeviceId = deviceId ?? uuidv4();
    
    // Make sure time is an integer
    const timestamp = Math.floor(new Date().getTime());
    
    // Server-specific values
    let pid = "";
    let lang = "";
    
    if (server === "en") {
        pid = "US-ARKNIGHTS";
        lang = "en";
    } else if (server === "jp") {
        pid = "JP-AK";
        lang = "ja";
    } else if (server === "kr") {
        pid = "KR-ARKNIGHTS";
        lang = "ko";
    }
    
    // Create the map with values
    const linkedHashMap = {
        Channel: "googleplay",
        DeviceID: actualDeviceId,
        DeviceModel: "F9",
        GBuildNo: "",
        GVersionNo: "2000112",
        Lang: lang,
        PID: pid,
        Platform: "android",
        Time: timestamp,
        Token: yostarToken ?? "",
        UID: yostarUID ?? "",
        Version: "4.10.0"
    };
    
    // Manually create a JSON string to match Python exactly
    // The order here is alphabetical (Python's default for sorted keys)
    const jsonString = `{"Channel":"googleplay","DeviceID":"${actualDeviceId}","DeviceModel":"F9","GBuildNo":"","GVersionNo":"2000112","Lang":"${lang}","PID":"${pid}","Platform":"android","Time":${timestamp},"Token":"${yostarToken ?? ""}","UID":"${yostarUID ?? ""}","Version":"4.10.0"}`;
    
    // SHA1 salt used by Yostar's servers
    const secretKey = "886c085e4a8d30a703367b120dd8353948405ec2";
    
    // Create the string to hash: jsonString + body + secretKey
    const stringToHash = jsonString + body + secretKey;
    
    // Calculate MD5 hash with UTF-8 encoding
    const md5Hash = createHash("md5")
        .update(stringToHash, "utf8")
        .digest("hex")
        .toUpperCase();
    
    // Create the Authorization header - keeping the original map
    const headerAuth = {
        Head: linkedHashMap,
        Sign: md5Hash
    };
    
    // Format the header as a JSON string
    return {
        Authorization: JSON.stringify(headerAuth),
        "Content-Type": "application/json",
    };
};
