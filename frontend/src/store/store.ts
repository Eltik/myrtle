import { create } from "zustand";
import { persist } from "zustand/middleware";
import { type LoginResponse } from "~/types/types";

// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call
export const useLogin = create(
    // eslint-disable-next-line @typescript-eslint/no-unsafe-call
    persist(
        (set, get) => ({
            loginData: {
                channelUID: null,
                token: null,
                uid: null,
                secret: null,
                seqnum: null
            },
            // eslint-disable-next-line @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-call
            setLogin: (loginData: LoginResponse) => set({ loginData }),
        }),
        {
            name: "loginData",
        }
    )
);