import type { CharacterData } from "~/types/impl/api";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../ui/card";
import { Progress } from "../ui/progress";
import Image from "next/image";
import { Dialog, DialogContent, DialogTrigger } from "../ui/dialog";
import CharacterDialogueCard from "./character-dialogue-card";

function CharacterCard({ data }: { data: CharacterData }) {
    return (
        <Dialog>
            <DialogTrigger asChild>
                <Card className="cursor-pointer transition-all duration-150 hover:bg-secondary">
                    <CardHeader>
                        <CardTitle>{data.static?.name}</CardTitle>
                        <CardDescription>Level {data.level}</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div className="grid grid-cols-2">
                            <div className="relative h-full w-full">
                                <Image className="h-full w-full rounded-lg object-cover" alt="Operator Image" width={500} height={500} src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/characters/${encodeURIComponent(data.skin.includes("@") ? data.skin.replaceAll("@", "_") : data.skin.replaceAll("#", "_"))}.png`} />
                            </div>
                            <div>
                                <div className="flex justify-between">
                                    <span>Potential:</span>
                                    <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/potential/${data.potentialRank + 1}.png`} width={35} height={35} alt="Potential" />
                                </div>
                                <div className="flex justify-between">
                                    <span>Elite:</span>
                                    <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${data.evolvePhase}.png`} width={35} height={35} alt="Promotion" />
                                </div>
                            </div>
                        </div>
                        <div className="space-y-2">
                            <div className="flex flex-col">
                                <div className="flex justify-between">
                                    <span>Recruited:</span>
                                    <span className="text-sm text-muted-foreground">{new Date(data.gainTime * 1000).toLocaleString()}</span>
                                </div>
                                <div className="flex justify-between">
                                    <span>Language:</span>
                                    <span className="text-sm text-muted-foreground">{data.voiceLan}</span>
                                </div>
                            </div>
                            <Progress value={(data.static?.trust ?? 0) / 2} className="w-[60%]" />
                            <div className="text-sm text-muted-foreground">Trust: {data.static?.trust}%</div>
                        </div>
                    </CardContent>
                </Card>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[425px]">
                <CharacterDialogueCard data={data} />
            </DialogContent>
        </Dialog>
    );
}
export default CharacterCard;
