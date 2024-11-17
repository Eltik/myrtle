import type { User } from "..";

export type PlayerResponse = {
    id: string;
    uid: string;
    server: string;
    data: User;
};
