import type { User } from "./impl/api";

export type Cookies = {
    login: {
        uid: string;
        secret: string;
        seqnum: number;
    };
    playerData: User;
};
