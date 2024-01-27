import type { PlayerData } from "~/types/types";
import { Avatar, AvatarFallback, AvatarImage } from "./ui/avatar";
import { Separator } from "./ui/separator";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Dialog, DialogContent, DialogTrigger } from "./ui/dialog";
import CharacterCard from "./character-card";
import { getInitials } from "~/helper";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select";
import { useEffect, useState } from "react";

export function PlayerProfile({ data }: { data: PlayerData }) {
    const [sort, setSort] = useState("level");

    useEffect(() => {
        console.log(sort)
    }, [sort]);

    return (
        <div className="flex flex-col gap-6 md:flex-row md:gap-10">
            <div className="flex flex-col items-center gap-4 rounded-lg bg-white p-6 shadow-md dark:bg-accent">
                <Avatar className="h-24 w-24 border-4 border-gray-200 bg-muted dark:border-gray-700">
                    <AvatarImage
                        src={
                            data.status?.avatarId
                                ? data.status.avatar.type === "ASSISTANT"
                                    ? `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${
                                          Object.values(data.troop.chars)
                                              .find((item) => item.skin === data.status?.avatar.id ?? "")
                                              ?.charId.replaceAll("#", "_") ?? ""
                                      }.png`
                                    : ""
                                : "https://static.wikia.nocookie.net/mrfz/images/4/46/Symbol_profile.png/revision/latest?cb=20220418145951"
                        }
                        alt="@shadcn"
                    />
                    <AvatarFallback>{getInitials(data.status?.nickName?.slice(0, 1))}</AvatarFallback>
                </Avatar>
                <h2 className="text-xl font-bold">
                    {data.status?.nickName}#{data.status?.nickNumber}
                </h2>
                <div className="flex flex-col">
                    <span className="text-sm text-gray-500 dark:text-gray-400">
                        <b>Level</b> {data.status?.level}
                    </span>
                    <span className="text-sm text-gray-500 dark:text-gray-400">
                        <b>Playing Since:</b> {`${new Date(data.status?.registerTs * 1000).getFullYear()}/${new Date(data.status?.registerTs * 1000).getMonth()}/${new Date(data.status?.registerTs * 1000).getDate()}`}
                    </span>
                    <span className="text-sm text-gray-500 dark:text-gray-400">
                        <b>Current Stage:</b> {data.status?.mainStageProgress}
                    </span>
                </div>
                <Separator className="bg-gray-400" />
            </div>
            <div className="flex-1">
                <Card>
                    <CardHeader>
                        <div className="space-y-4">
                            <div className="flex flex-row justify-between items-center">
                                <CardTitle>Operators</CardTitle>
                                <Select value={sort} onValueChange={setSort}>
                                    <SelectTrigger className="w-[180px]">
                                        <SelectValue placeholder="Level" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="level">Level</SelectItem>
                                        <SelectItem value="trust">Trust</SelectItem>
                                        <SelectItem value="obtained">Obtained Time</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>
                            <Separator />
                        </div>
                    </CardHeader>
                    <CardContent>
                        <div className="grid max-h-96 gap-4 overflow-y-scroll">
                            {data.troop?.chars
                                ? (sort === "level" ?
                                    Object.values(data.troop.chars)
                                        .sort((a, b) => b.level - a.level)
                                        .sort((a, b) => b.evolvePhase - a.evolvePhase) :
                                    sort === "trust" ? 
                                    Object.values(data.troop.chars)
                                        .sort((a, b) => b.favorPoint - a.favorPoint) :
                                    sort === "obtained" ?
                                        Object.values(data.troop.chars)
                                        .sort((a, b) => a.gainTime - b.gainTime) :
                                    Object.values(data.troop.chars)
                                        .sort((a, b) => b.level - a.level)
                                        .sort((a, b) => b.evolvePhase - a.evolvePhase))
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
    );
}
