import type { User } from "..";

export type PlayerResponse = UserDB[] & {
    error?: string;
    message?: string;
};

export type UserDB = {
    id: string;
    uid: string;
    server: string;
    data: User;
};
