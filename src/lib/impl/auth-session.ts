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
