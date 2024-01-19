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
            <main className="bg-main-blue-200 h-screen">
                <Navbar />
                <div className="px-4 py-24">
                    <div className="flex">
                        <ClientOnly>
                            <main className="flex-grow bg-[#262626] rounded-lg p-4">
                                <div className="flex justify-between items-center mb-4">
                                    <span className="text-white text-lg">Hired *some time here*</span>
                                    <div className="flex items-center gap-3">
                                        <div className="flex flex-col items-center">
                                            <span className="relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full">
                                                <Image className="aspect-square h-full w-full" alt="Player Avatar" src={Number(playerData.status?.avatarId) === 0 ? "https://static.wikia.nocookie.net/mrfz/images/4/46/Symbol_profile.png/revision/latest?cb=20220418145951" :`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${playerData.status?.avatarId}`} width={40} height={40} />
                                            </span>
                                            <div className="bg-[#333333] text-white mt-2 px-3 py-1 rounded-full">Dr. {playerData.status?.nickName}</div>
                                        </div>
                                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-white mr-2">
                                            <circle cx="12" cy="12" r="10" />
                                            <path d="M12 16v-4" />
                                            <path d="M12 8h.01" />
                                        </svg>
                                    </div>
                                </div>
                                <div className="flex items-center mb-4 gap-3">
                                    <div className="rounded-md object-cover" style={{
                                        backgroundImage: `url("https://gamepress.gg/sites/arknights/files/2019-10/bg.png")`,
                                        backgroundSize: "cover",
                                        backgroundPosition: "center",
                                        aspectRatio: "16/9",
                                    }}>
                                        <Image alt="Assistant" className="rounded-md object-cover" src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/characters/${playerData.status?.secretarySkinId.replaceAll("#", "_") ?? ""}.png`} width={300} height={200} style={{
                                            aspectRatio: "200/200",
                                        }} />
                                    </div>
                                    <div className="flex-grow ml-4">
                                        <div className="text-white text-lg mb-2">{playerData.status?.secretary}</div>
                                        <div className="bg-[#333333] text-white rounded-lg p-2 mb-4">Current Stage: {playerData.status?.mainStageProgress}</div>
                                        <div className="bg-[#333333] text-white rounded-lg p-2">Furniture Owned 588</div>
                                    </div>
                                </div>
                                <div className="grid grid-cols-2 gap-3">
                                    <div>
                                        <div className="flex items-center justify-between bg-[#333333] text-white rounded-lg p-2 mb-4">
                                            <div>Operators Hired</div>
                                            <div>{playerData.troop?.curSquadCount ?? 0}</div>
                                        </div>
                                        <div className="grid grid-cols-4 gap-4" />
                                    </div>
                                    <div>
                                        <div className="bg-[#333333] text-white rounded-lg p-2 mb-4">
                                            <div className="flex items-center justify-between">
                                                <div>Something Here</div>
                                                {/*<SearchIcon className="text-white" />*/}
                                            </div>
                                            <div className="grid grid-cols-2 gap-4" />
                                        </div>
                                        <div className="flex overflow-x-auto space-x-2 pb-2" />
                                    </div>
                                </div>
                            </main>
                        </ClientOnly>
                    </div>
                </div>
            </main>
        </>
    )
}