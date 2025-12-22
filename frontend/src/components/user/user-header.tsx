"use client";

import { Clipboard } from "lucide-react";
import { toast } from "sonner";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { InView } from "~/components/ui/motion-primitives/in-view";
import type { User } from "~/types/api/impl/user";

// Helper function to get avatar URL from CDN
function getAvatarUrl(user: User): string {
    if (!user.status) return "/placeholder.svg?height=80&width=80";

    const secretaryId = user.status.secretary;
    const secretarySkinId = user.status.secretarySkinId;

    // Use base character ID for default skins (#1), otherwise use the skin ID
    const skinId = !secretarySkinId?.includes("@") && secretarySkinId?.endsWith("#1") ? secretaryId : secretarySkinId;

    if (!skinId) return "/placeholder.svg?height=80&width=80";

    // Normalize skin ID: replace @ and # with _
    const normalizedSkinId = skinId.replaceAll("@", "_").replaceAll("#", "_");

    // Use CDN for avatar - the backend will find the correct avatar directory
    return `/api/cdn/upk/spritepack/ui_char_avatar_0/${normalizedSkinId}.png`;
}

function Stat({ label, value }: { label: string; value: number }) {
    return (
        <div className="text-center">
            <div className="font-bold text-2xl">
                <AnimatedNumber value={value} springOptions={{ bounce: 0, duration: 1000 }} />
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
            variants={{
                hidden: { opacity: 0, y: -20 },
                visible: { opacity: 1, y: 0 },
            }}
            transition={{ duration: 0.5, ease: "easeOut" }}
        >
            <Card className="mx-auto mt-5 mb-8">
                <CardHeader>
                    <div className="flex items-center space-x-4">
                        <Avatar className="h-20 w-20">
                            <AvatarImage src={getAvatarUrl(data) || "/placeholder.svg"} alt={data.status?.nickName ?? "User"} />
                            <AvatarFallback>{data.status?.nickName?.slice(0, 1) ?? "E"}</AvatarFallback>
                        </Avatar>
                        <div>
                            <CardTitle className="text-2xl">
                                <div className="flex flex-row gap-4">
                                    <div>
                                        {data.status.nickName && data.status.nickName.length > 0 ? data.status.nickName : "Unknown"}
                                        <span className="text-muted-foreground">#{data.status.nickNumber}</span>
                                    </div>
                                    <div className="hidden cursor-pointer rounded-md border p-2 transition-all duration-150 hover:bg-secondary md:block" onClick={handleCopyUsername} onKeyDown={(e) => e.key === "Enter" && handleCopyUsername()} role="button" tabIndex={0}>
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
        </InView>
    );
}
