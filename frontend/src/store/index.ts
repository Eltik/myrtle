import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { User } from "~/types/impl/api";

export const usePlayer = create(
    persist(
        (set) => ({
            playerData: {},
            setLogin: (playerData: User) => set({ playerData }),
        }),
        {
            name: "playerData",
        },
    ),
);
