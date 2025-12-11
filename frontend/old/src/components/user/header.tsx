import type { User } from "~/types/impl/api";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../ui/card";
import { Avatar, AvatarFallback, AvatarImage } from "../ui/avatar";
import { toast } from "~/hooks/use-toast";
import { Clipboard } from "lucide-react";
import Stat from "./components/stat";
import { getAvatarSkinId } from "~/helper";

function UserHeader({ data }: { data: User }) {
    return (
        <Card className="mx-auto mb-8 mt-5">
            <CardHeader>
                <div className="flex items-center space-x-4">
                    <Avatar className="h-20 w-20">
                        <AvatarImage src={getAvatarSkinId(data)} alt="@shadcn" />
                        <AvatarFallback>{data.status?.nickName?.slice(0, 1) ?? "E"}</AvatarFallback>
                    </Avatar>
                    <div>
                        <CardTitle className="text-2xl">
                            <div className="flex flex-row gap-4">
                                <div>
                                    {data.status.nickName && data.status.nickName.length > 0 ? data.status.nickName : "Unknown"}
                                    <span className="text-muted-foreground">#{data.status.nickNumber}</span>
                                </div>
                                <div
                                    className="hidden cursor-pointer rounded-md border p-2 transition-all duration-150 hover:bg-secondary md:block"
                                    onClick={() => {
                                        void navigator.clipboard.writeText(`${data.status.nickName && data.status.nickName.length > 0 ? data.status.nickName : "Unknown"}#${data.status.nickNumber}`);
                                        toast({
                                            title: "Success!",
                                            description: "Copied username to clipboard.",
                                        });
                                    }}
                                >
                                    <Clipboard size={15} />
                                </div>
                            </div>
                        </CardTitle>
                        <CardDescription>
                            <div className="flex flex-col">
                                <span>Level {data.status.level}</span>
                                <span>{data.status.resume}</span>
                            </div>
                        </CardDescription>
                    </div>
                </div>
            </CardHeader>
            <CardContent>
                <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
                    <Stat label="LMD" value={data.status.gold} />
                    <Stat label="Orundum" value={data.status.diamondShard} />
                    <Stat label="Originium" value={data.status.payDiamond + data.status.freeDiamond} />
                    <Stat label="Friend Limit" value={data.status.friendNumLimit} />
                </div>
            </CardContent>
        </Card>
    );
}

export default UserHeader;
