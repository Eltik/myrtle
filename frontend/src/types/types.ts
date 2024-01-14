export type Server = "en" | "jp" | "kr" | "cn" | "bili" | "tw";

export type CodeResponse = {
    result: number;
};

export type LoginResponse = {
    channelUID: string;
    token: string;
    uid: string;
    secret: string;
    seqnum: number;
};