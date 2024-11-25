export type LoginResponse = {
    uid?: string;
    secret?: string;
    seqnum?: number;

    message?: string;
    error?: string;
};
