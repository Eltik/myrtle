import Image from "next/image";
import { useEffect } from "react";
import { useStore } from "zustand";
import Navbar from "~/components/navbar";
import ClientOnly from "~/lib/ClientOnly";
import { useLogin, usePlayer } from "~/store/store";
import { type LoginData, type PlayerData } from "~/types/types";

export default function Profile() {
    const loginData = useStore(useLogin, (state) => (state as { loginData: LoginData })?.loginData);
    const playerData = useStore(usePlayer, (state) => (state as { playerData: PlayerData })?.playerData);

    const refresh = async() => {
        // Refresh ig
    };

    return (
        <>
            <main className="bg-main-blue-200">
                <Navbar />
                <ClientOnly>
                    <div className="px-4 py-24">
                        <div className="flex flex-col h-screen bg-[#000000] text-white rounded-md">
                            <header className="flex items-center justify-between px-6 py-4 border-b border-gray-700">
                                <h1 className="text-2xl font-bold">Arknights</h1>
                                <div className="flex items-center gap-4">
                                    <button type="button" className="">
                                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-4 w-4">
                                            <path d="M2 20h.01" />
                                            <path d="M7 20v-4" />
                                            <path d="M12 20v-8" />
                                            <path d="M17 20V8" />
                                            <path d="M22 4v16" />
                                        </svg>
                                        <span className="sr-only">Notifications</span>
                                    </button>
                                    <button type="button" className="">
                                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-4 w-4">
                                            <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
                                            <circle cx="12" cy="12" r="3" />
                                        </svg>
                                        <span className="sr-only">Settings</span>
                                    </button>
                                </div>
                            </header>
                            <main className="flex flex-col-reverse md:flex-1 md:flex-row overflow-hidden">
                                <section className="w-1/4 border-r border-gray-700 p-6">
                                    <h2 className="text-xl font-bold mb-4">{playerData.status?.nickName}#{playerData.status?.nickNumber}</h2>
                                    <div className="flex flex-col gap-4">
                                        <div className="grid grid-cols-2 gap-4">
                                            <div className="flex flex-col gap-1">
                                                <span>Current Stage</span>
                                                <span>Stage 1</span>
                                            </div>
                                            <div className="flex flex-col gap-1">
                                                <span>Operators Hired</span>
                                                <span>50/50</span>
                                            </div>
                                            <div className="flex flex-col gap-1">
                                                <span>Operators</span>
                                                <span>2000</span>
                                            </div>
                                            <div className="flex flex-col gap-1">
                                                <span>Furniture</span>
                                                <span>1000</span>
                                            </div>
                                        </div>
                                    </div>
                                </section>
                                <section className="w-3/4 rounded-br-md">
                                    <div className="object-cover rounded-br-md" style={{
                                        backgroundImage: `url("https://gamepress.gg/sites/arknights/files/2019-10/bg.png")`,
                                        backgroundSize: "cover",
                                        backgroundPosition: "center",
                                        aspectRatio: "16/9",
                                    }}>
                                        <div className="flex flex-col lg:flex-row w-full h-full">
                                            <Image alt="Assistant" className="object-cover" src={playerData.status?.secretarySkinId.length > 0 ? `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/characters/${playerData.status?.secretarySkinId.replaceAll("#", "_") ?? ""}.png` : "https://static.wikia.nocookie.net/mrfz/images/4/46/Symbol_profile.png/revision/latest?cb=20220418145951"} width={800} height={800} style={{
                                                aspectRatio: "200/200",
                                            }} />
                                            <div className="w-full h-full">
                                                <div className="rounded-lg border shadow-sm max-w-xl bg-gray-800 text-white">
                                                    <div className="flex-col space-y-1.5 flex p-4 border-b border-gray-700">
                                                        <div className="flex items-center space-x-4">
                                                            <span className="relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full">
                                                                <Image className="aspect-square h-full w-full" alt="Player Avatar" src={playerData.status?.avatarId ? (playerData.status.avatar.type === "ASSISTANT" ? `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${(Object.values(playerData.troop.chars).find((item) => item.skin === playerData.status?.avatar.id ?? ""))?.charId.replaceAll("#", "_") ?? ""}.png` : "") : "https://static.wikia.nocookie.net/mrfz/images/4/46/Symbol_profile.png/revision/latest?cb=20220418145951"} width={40} height={40} />
                                                            </span>
                                                            <div className="flex flex-col">
                                                                <span className="text-lg font-bold">Dr. {playerData.status?.nickName}#{playerData.status?.nickNumber}</span>
                                                                <span className="text-sm opacity-70"><i>{playerData.status?.resume.length > 0 ? playerData.status?.resume : "This lazy individual didn't leave a message."}</i></span>
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <div className="p-4">
                                                        <div className="grid grid-cols-2 gap-4">
                                                            {playerData.social?.assistCharList.map((item, index) => {
                                                                if (!item) return;
                                                                return (
                                                                    <div key={`assistant-${index}`} className="rounded-lg border text-card-foreground shadow-sm bg-gray-700">
                                                                        <div className="w-24 h-24" style={{
                                                                            backgroundImage: (Object.values(playerData.troop?.chars).filter((char) => char.instId === (item?.charInstId))[0]?.charId ?? "").length > 0 ? `url("https://raw.githubusercontent.com/Aceship/Arknight-Images/main/avatars/${(Object.values(playerData.troop?.chars).filter((char) => char.instId === (item?.charInstId))[0]?.charId ?? "").replaceAll("#", "_")}.png")` : "https://static.wikia.nocookie.net/mrfz/images/4/46/Symbol_profile.png/revision/latest?cb=20220418145951",
                                                                            backgroundSize: "cover",
                                                                            backgroundPosition: "center",
                                                                            aspectRatio: "1/1",
                                                                        }}>
                                                                        </div>
                                                                    </div>
                                                                )
                                                            })}
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </section>
                            </main>
                        </div>
                    </div>
                </ClientOnly>
            </main>
        </>
    )
}