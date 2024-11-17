import type { User } from "../../lib/impl/user/impl/get";

export type UserDB = {
    id: string;
    uid: string;
    server: string;
    data: User;
    created_at: string;
};
