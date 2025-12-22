"use client";

import { Clipboard } from "lucide-react";
import { toast } from "sonner";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { getSecretaryAvatarUrl } from "~/lib/utils";
import type { User } from "~/types/api/impl/user";

function Stat({ label, value }: { label: string; value: number }) {
    return (
        <div className="text-center">
            <div className="font-bold text-2xl">
                <AnimatedNumber springOptions={{ bounce: 0, duration: 1000 }} value={value} />
            </div>
            <div className="text-muted-foreground text-sm">{label}</div>
        </div>
    );
}

interface UserHeaderProps {
    data: User;
}

export function UserHeader({ data }: UserHeaderProps) {
    const handleCopyUsername = () => {
        const username = `${data.status.nickName && data.status.nickName.length > 0 ? data.status.nickName : "Unknown"}#${data.status.nickNumber}`;
        void navigator.clipboard.writeText(username);
        toast.success("Copied username to clipboard!");
    };

    return (
        <InView
            transition={{ duration: 0.5, ease: "easeOut" }}
            variants={{
                hidden: { opacity: 0, y: -20 },
                visible: { opacity: 1, y: 0 },
            }}
        >
            <Card className="mx-auto mt-5 mb-8">
                <CardHeader>
                    <div className="flex items-center space-x-4">
                        <Avatar className="h-20 w-20">
                            <AvatarImage alt={data.status?.nickName ?? "User"} src={getSecretaryAvatarUrl(data)} />
                            <AvatarFallback>{data.status?.nickName?.slice(0, 1) ?? "E"}</AvatarFallback>
                        </Avatar>
                        <div>
                            <CardTitle className="text-2xl">
                                <div className="flex flex-row gap-4">
                                    <div>
                                        {data.status.nickName && data.status.nickName.length > 0 ? data.status.nickName : "Unknown"}
                                        <span className="text-muted-foreground">#{data.status.nickNumber}</span>
                                    </div>
                                    <button className="hidden cursor-pointer rounded-md border p-2 transition-all duration-150 hover:bg-secondary md:block" onClick={handleCopyUsername} onKeyDown={(e) => e.key === "Enter" && handleCopyUsername()} tabIndex={0} type="button">
                                        <Clipboard size={15} />
                                    </button>
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
        </InView>
    );
}
