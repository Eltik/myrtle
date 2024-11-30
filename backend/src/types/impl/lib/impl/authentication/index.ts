export type AKDistributor = "yostar" | "hypergryph" | "bilibili" | "longcheng";
export type AKServer = "en" | "jp" | "kr" | "cn" | "bili" | "tw";
export type AKDomain = "gs" | "as" | "u8" | "hu" | "hv" | "rc" | "an" | "prean" | "sl" | "of" | "pkgAd" | "pkgIOS";

export class AuthSession {
    public uid: string; // uid of the player
    public secret: string; // secret of the player
    // sequence number based on the requests sent from the same
    // session. For example, if the first request sent from this
    // session must be 1 or higher, the second request must be 2 or higher, and so on.
    // If you encounter errors saying "stale sequence number", try incrementing this number.
    public seqnum: number = 1;

    constructor(uid?: string, secret?: string, seqnum?: number) {
        this.uid = uid || "";
        this.secret = secret || "";
        this.seqnum = seqnum || 1;
    }
}
