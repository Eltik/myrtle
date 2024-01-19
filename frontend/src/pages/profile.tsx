import Image from "next/image";
import { useStore } from "zustand";
import Navbar from "~/components/navbar";
import ClientOnly from "~/lib/ClientOnly";
import { useLogin, usePlayer } from "~/store/store";
import { type LoginData, type PlayerData } from "~/types/types";

export default function Profile() {
    const loginData = useStore(useLogin, (state) => (state as { loginData: LoginData })?.loginData);
    const playerData = useStore(usePlayer, (state) => (state as { playerData: PlayerData })?.playerData);

    return (
        <>
            <main className="bg-main-blue-200">
                <Navbar />
                <div className="px-4 py-4">
                    <div className="flex">
                        <aside className="w-64">
                            <ul className="space-y-2">
                                <li>
                                    <button type="button" className="inline-flex h-10 items-center justify-start rounded-md px-8 text-sm font-medium shadow transition-colors focus-visible:outline-none focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50 bg-gray-50 text-gray-900 hover:bg-gray-50/90 focus-visible:ring-gray-300">
                                        Profile
                                    </button>
                                </li>
                                <li>
                                    <button type="button" className="inline-flex h-10 items-center justify-start rounded-md px-8 text-sm font-medium shadow transition-colors focus-visible:outline-none focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50 bg-gray-50 text-gray-900 hover:bg-gray-50/90 focus-visible:ring-gray-300">
                                        Friends
                                    </button>
                                </li>
                                <li>
                                    <button type="button" className="inline-flex h-10 items-center justify-start rounded-md px-8 text-sm font-medium shadow transition-colors focus-visible:outline-none focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50 bg-gray-50 text-gray-900 hover:bg-gray-50/90 focus-visible:ring-gray-300">
                                        Add
                                    </button>
                                </li>
                                <li>
                                    <button type="button" className="inline-flex h-10 items-center justify-start rounded-md px-8 text-sm font-medium shadow transition-colors focus-visible:outline-none focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50 bg-gray-50 text-gray-900 hover:bg-gray-50/90 focus-visible:ring-gray-300">
                                        Support Unit
                                    </button>
                                </li>
                            </ul>
                        </aside>
                        <main className="flex-1 ml-4">
                            <div className="bg-gray-700 p-4 rounded-lg">
                                <div className="flex items-center mb-4">
                                    <span className="relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full">
                                        {/*<Image className="aspect-square h-full w-full" src={""} />*/}
                                    </span>
                                    <ClientOnly>
                                        <div className="ml-4">
                                            <div className="text-2xl">{playerData.status?.nickName}# {playerData.status?.nickNumber}</div>
                                            <div className="text-sm text-gray-400">Hired </div>
                                            <div className="text-sm text-gray-400">ASSISTANT</div>
                                        </div>
                                    </ClientOnly>
                                </div>
                            </div>
                        </main>
                    </div>
                </div>
            </main>
        </>
    )
}