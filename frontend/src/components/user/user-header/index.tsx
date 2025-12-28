"use client";

import { Check, Clipboard } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";
import { toast } from "sonner";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { getSecretaryAvatarUrl } from "~/lib/utils";
import type { User } from "~/types/api/impl/user";
import { Stat } from "./impl/stat";

interface UserHeaderProps {
    data: User;
}

export function UserHeader({ data }: UserHeaderProps) {
    const [isCopied, setIsCopied] = useState(false);

    const handleCopyUsername = () => {
        const username = `${data.status.nickName && data.status.nickName.length > 0 ? data.status.nickName : "Unknown"}#${data.status.nickNumber}`;
        void navigator.clipboard.writeText(username);
        toast.success("Copied username to clipboard!");
        setIsCopied(true);
        setTimeout(() => setIsCopied(false), 2000);
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
                                    <motion.button
                                        animate={{ scale: isCopied ? [1, 0.85, 1] : 1 }}
                                        className="relative hidden size-[34px] cursor-pointer items-center justify-center rounded-md border transition-colors duration-150 hover:bg-secondary md:flex"
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
