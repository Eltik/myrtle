import { useState } from "react";
import { useStore } from "zustand";
import Navbar from "~/components/navbar";
import { PlayerProfile } from "~/components/player-profile";
import { Button } from "~/components/ui/button";
import { useToast } from "~/components/ui/use-toast";
import ClientOnly from "~/lib/ClientOnly";
import { useLogin, usePlayer } from "~/store/store";
import { type Server, type LoginData, type PlayerData } from "~/types/types";

export default function Profile() {
    const loginData = useStore(useLogin, (state) => (state as { loginData: LoginData })?.loginData);
    const playerData = useStore(usePlayer, (state) => (state as { playerData: PlayerData })?.playerData);

    const [server, setServer] = useState<Server>("en");

    const { toast } = useToast();

    const refresh = async () => {
        try {
            const playerData = (await (
                await fetch("/api/refresh", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        uid: loginData.uid,
                        email: loginData.email,
                        secret: loginData.secret,
                        seqnum: loginData.seqnum++,
                        server,
                    }),
                })
            ).json()) as PlayerData;
            if ((playerData as unknown as { statusCode: number }).statusCode === 400) {
                useLogin.setState({ loginData: { ...loginData, seqnum: (loginData.seqnum += 2) } });
                return window.location.reload();
            }

            // Inccrease seqnum
            useLogin.setState({ loginData: { ...loginData, seqnum: loginData.seqnum++ } });

            if ((playerData as unknown as { statusCode: number }).statusCode === 401) {
                // Clear login data
                useLogin.setState({ loginData: null });
            } else {
                console.log("Fetched player data successfully.");
                usePlayer.setState({ playerData });
                window.location.reload();
            }
        } catch (e) {
            toast({
                title: "Uh oh! There was an error!",
                description: "The server was unable to fetch player information from your account. Please refresh and try again later.",
            });
        }
    };

    return (
        <>
            <main>
                <Navbar />
                <ClientOnly>
                    <div className="flex flex-1 flex-col gap-6 p-4 md:p-10">
                        <Button variant={"outline"} onClick={() => refresh()}>
                            Refresh
                        </Button>
                        <PlayerProfile data={playerData} />
                    </div>
                </ClientOnly>
            </main>
        </>
    );
}
