import { create } from "zustand";
import { persist } from "zustand/middleware";
import { type PlayerData, type LoginData } from "~/types/types";

export const usePlayer = create(
    persist(
        (set, get) => ({
            playerData: {},
            setLogin: (playerData: PlayerData) => set({ playerData }),
        }),
        {
            name: "playerData",
        },
    ),
);

export const useLogin = create(
    persist(
        (set, get) => ({
            loginData: {
                channelUID: null,
                email: null,
                token: null,
                uid: null,
                secret: null,
                seqnum: null,
            },
            setLogin: (loginData: LoginData) => set({ loginData }),
        }),
        {
            name: "loginData",
        },
    ),
);
