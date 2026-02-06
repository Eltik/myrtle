"use client";

import { Check, Clipboard } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";
import { toast } from "sonner";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { getSecretaryAvatarURL } from "~/lib/utils";
import { Stat } from "./impl/stat";

interface UserProfile {
    nickName: string;
    nickNumber: string;
    level: number;
    resume: string;
    avatarId: string;
    secretary: string;
    secretarySkinId: string;
    gold: number;
    diamondShard: number;
    payDiamond: number;
    freeDiamond: number;
    registerTs: number;
}

interface UserHeaderProps {
    profile: UserProfile;
}

export function UserHeader({ profile }: UserHeaderProps) {
    const [isCopied, setIsCopied] = useState(false);

    const handleCopyUsername = () => {
        const username = `${profile.nickName && profile.nickName.length > 0 ? profile.nickName : "Unknown"}#${profile.nickNumber}`;
        void navigator.clipboard.writeText(username);
        toast.success("Copied username to clipboard!");
        setIsCopied(true);
        setTimeout(() => setIsCopied(false), 2000);
    };

    // Create a compatible object for getSecretaryAvatarURL
    const avatarData = { status: { secretary: profile.secretary, secretarySkinId: profile.secretarySkinId } };

    return (
        <InView
            once
            transition={{ duration: 0.5, ease: "easeOut" }}
            variants={{
                hidden: { opacity: 0, y: -20 },
                visible: { opacity: 1, y: 0 },
            }}
        >
            <Card className="mx-auto mt-5 mb-8">
                <CardHeader>
                    <div className="flex items-center space-x-4 overflow-hidden">
                        <Avatar className="h-20 w-20">
                            <AvatarImage alt={profile.nickName ?? "User"} src={getSecretaryAvatarURL(avatarData)} />
                            <AvatarFallback>{profile.nickName?.slice(0, 1) ?? "E"}</AvatarFallback>
                        </Avatar>
                        <div className="min-w-0 flex-1">
                            <CardTitle className="min-w-0 text-2xl">
                                <div className="flex min-w-0 flex-row gap-2 md:gap-4">
                                    <button
                                        className="cursor-pointer truncate active:opacity-70 md:cursor-default md:active:opacity-100"
                                        onClick={() => {
                                            if (window.matchMedia("(max-width: 767px)").matches) {
                                                handleCopyUsername();
                                            }
                                        }}
                                        onKeyDown={(e) => {
                                            if (e.key === "Enter" && window.matchMedia("(max-width: 767px)").matches) {
                                                handleCopyUsername();
                                            }
                                        }}
                                        tabIndex={0}
                                        type="button"
                                    >
                                        {profile.nickName && profile.nickName.length > 0 ? profile.nickName : "Unknown"}
                                        <span className="text-muted-foreground">#{profile.nickNumber}</span>
                                    </button>
                                    <motion.button
                                        animate={{ scale: isCopied ? [1, 0.85, 1] : 1 }}
                                        className="relative hidden size-8.5 cursor-pointer items-center justify-center rounded-md border transition-colors duration-150 hover:bg-secondary md:flex"
                                        onClick={handleCopyUsername}
                                        onKeyDown={(e) => e.key === "Enter" && handleCopyUsername()}
                                        tabIndex={0}
                                        transition={{ duration: 0.15, ease: "easeOut" }}
                                        type="button"
                                    >
                                        <AnimatePresence mode="wait">
                                            {isCopied ? (
                                                <motion.div animate={{ opacity: 1, scale: 1, filter: "blur(0px)" }} exit={{ opacity: 0, scale: 0.6, filter: "blur(3px)" }} initial={{ opacity: 0, scale: 0.6, filter: "blur(3px)" }} key="check" transition={{ duration: 0.12, ease: "easeOut" }}>
                                                    <Check size={15} />
                                                </motion.div>
                                            ) : (
                                                <motion.div animate={{ opacity: 1, scale: 1, filter: "blur(0px)" }} exit={{ opacity: 0, scale: 0.6, filter: "blur(3px)" }} initial={{ opacity: 0, scale: 0.6, filter: "blur(3px)" }} key="clipboard" transition={{ duration: 0.12, ease: "easeOut" }}>
                                                    <Clipboard size={15} />
                                                </motion.div>
                                            )}
                                        </AnimatePresence>
                                    </motion.button>
                                </div>
                            </CardTitle>
                            <CardDescription>
                                <div className="flex flex-col">
                                    <span>Level {profile.level}</span>
                                    <span>{profile.resume}</span>
                                </div>
                            </CardDescription>
                        </div>
                    </div>
                </CardHeader>
                <CardContent>
                    <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
                        <Stat label="LMD" value={profile.gold} />
                        <Stat label="Orundum" value={profile.diamondShard} />
                        <Stat label="Originium" value={profile.payDiamond + profile.freeDiamond} />
                        <div className="text-center">
                            <div className="font-bold text-2xl">{profile.registerTs > 0 ? new Date(profile.registerTs * 1000).toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" }) : "Unknown"}</div>
                            <div className="text-muted-foreground text-sm">Registered</div>
                        </div>
                    </div>
                </CardContent>
            </Card>
        </InView>
    );
}
