import { CheckIcon } from "lucide-react";
import { useState } from "react";
import { useStore } from "zustand";
import CharacterCard from "~/components/character-card";
import Navbar from "~/components/navbar";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/avatar";
import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Dialog, DialogContent, DialogTrigger } from "~/components/ui/dialog";
import { Separator } from "~/components/ui/separator";
import { useToast } from "~/components/ui/use-toast";
import ClientOnly from "~/lib/ClientOnly";
import { useLogin, usePlayer } from "~/store/store";
import { type Server, type LoginData, type PlayerData } from "~/types/types";

export default function Profile() {
    const loginData = useStore(useLogin, (state) => (state as { loginData: LoginData })?.loginData);
    const playerData = useStore(usePlayer, (state) => (state as { playerData: PlayerData })?.playerData);

    const [server, setServer] = useState<Server>("en");

    const { toast } = useToast();

    const refresh = async() => {
        try {
            const playerData = (await (
                await fetch("/api/player", {
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
                useLogin.setState({ loginData: { ...loginData, seqnum: loginData.seqnum += 2 } });
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
                        <div className="flex flex-col gap-6 md:flex-row md:gap-10">
                            <div className="flex flex-col items-center gap-4 rounded-lg bg-white p-6 shadow-md dark:bg-accent">
                                <Avatar className="h-24 w-24 border-4 border-gray-200 bg-muted dark:border-gray-700">
                                    <AvatarImage
                                        src={
                                            playerData.status?.avatarId
                                                ? playerData.status.avatar.type === "ASSISTANT"
                                                    ? `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${
                                                        Object.values(playerData.troop.chars)
                                                            .find((item) => item.skin === playerData.status?.avatar.id ?? "")
                                                            ?.charId.replaceAll("#", "_") ?? ""
                                                    }.png`
                                                    : ""
                                                : "https://static.wikia.nocookie.net/mrfz/images/4/46/Symbol_profile.png/revision/latest?cb=20220418145951"
                                        }
                                        alt="@shadcn"
                                    />
                                    <AvatarFallback>{playerData.status?.nickName?.slice(0, 1) ?? "E"}</AvatarFallback>
                                </Avatar>
                                <h2 className="text-xl font-bold">
                                    {playerData.status?.nickName}#{playerData.status?.nickNumber}
                                </h2>
                                <div className="flex flex-col">
                                    <span className="text-sm text-gray-500 dark:text-gray-400">
                                        <b>Level</b> {playerData.status?.level}
                                    </span>
                                    <span className="text-sm text-gray-500 dark:text-gray-400">
                                        <b>Playing Since:</b> {`${new Date(playerData.status?.registerTs * 1000).getFullYear()}/${new Date(playerData.status?.registerTs * 1000).getMonth()}/${new Date(playerData.status?.registerTs * 1000).getDate()}`}
                                    </span>
                                    <span className="text-sm text-gray-500 dark:text-gray-400">
                                        <b>Current Stage:</b> {playerData.status?.mainStageProgress}
                                    </span>
                                </div>
                                <Separator className="bg-gray-400" />
                            </div>
                            <div className="flex-1">
                                <Card>
                                    <CardHeader>
                                        <div className="space-y-4">
                                            <CardTitle>Operators</CardTitle>
                                            <Separator />
                                        </div>
                                    </CardHeader>
                                    <CardContent>
                                        <div className="grid max-h-96 gap-4 overflow-y-scroll">
                                            {playerData.troop?.chars
                                                ? Object.values(playerData.troop.chars)
                                                    .sort((a, b) => b.level - a.level)
                                                    .map((character, index) => (
                                                        <>
                                                        <Dialog>
                                                            <DialogTrigger>
                                                                <div className="flex cursor-pointer items-center gap-4 rounded-md border-b-2 px-2 py-2 transition-all duration-150 ease-in-out hover:bg-accent" key={`operator-${index}`}>
                                                                    <Avatar className="h-10 w-10">
                                                                        <AvatarImage alt={`${character.static.name} Avatar`} src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/avatars/${character.charId.replaceAll("#", "_")}.png`} />
                                                                        <AvatarFallback>{character.static.name.slice(0, 1)}</AvatarFallback>
                                                                    </Avatar>
                                                                    <div className="grid flex-1 gap-0.5 text-left">
                                                                        <div className="font-medium">{character.static.name}</div>
                                                                        <div className="text-xs text-gray-500 dark:text-gray-400">Level {character.level}</div>
                                                                    </div>
                                                                    <div className="text-right text-sm">HP: 2000 | ATK: 1500</div>
                                                                </div>
                                                            </DialogTrigger>
                                                            <DialogContent className="sm:max-w-[425px]">
                                                                <CharacterCard data={character} />
                                                            </DialogContent>
                                                        </Dialog>
                                                        </>
                                                    ))
                                                : null}
                                        </div>
                                    </CardContent>
                                </Card>
                            </div>
                        </div>
                        <Card className="">
                            <CardHeader>
                                <CardTitle>Achievements</CardTitle>
                            </CardHeader>
                            <CardContent>
                                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                                    <div className="flex items-center gap-4">
                                        <Badge className="items-center" variant="outline">
                                            <CheckIcon className="h-4 w-4" />
                                            Achievement 1
                                        </Badge>
                                    </div>
                                    <div className="flex items-center gap-4">
                                        <Badge className="items-center" variant="outline">
                                            <CheckIcon className="h-4 w-4" />
                                            Achievement 2
                                        </Badge>
                                    </div>
                                    <div className="flex items-center gap-4">
                                        <Badge className="items-center" variant="outline">
                                            <CheckIcon className="h-4 w-4" />
                                            Achievement 3
                                        </Badge>
                                    </div>
                                </div>
                            </CardContent>
                        </Card>
                    </div>
                </ClientOnly>
            </main>
        </>
    );
}
