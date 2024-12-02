import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { StoredUser } from "~/types/impl/api";

export const usePlayer = create(
    persist(
        (set) => ({
            playerData: {},
            setLogin: (playerData: StoredUser) => set({ playerData }),
        }),
        {
            name: "playerData",
        },
    ),
);
