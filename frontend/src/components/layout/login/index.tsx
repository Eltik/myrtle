"use client";

import { X } from "lucide-react";
import { useState } from "react";
import { cn } from "~/lib/utils";
import { MorphingDialog, MorphingDialogClose, MorphingDialogContainer, MorphingDialogContent, MorphingDialogTrigger } from "../../ui/motion-primitives/morphing-dialog";
import { LoginContent } from "./impl/login-content";
import { LoginTriggerButton } from "./impl/login-trigger-button";
import type { LoginProps } from "./impl/types";

export function Login({ variant = "default" }: LoginProps) {
    const [open, setOpen] = useState(false);

    return (
        <MorphingDialog
            onOpenChange={setOpen}
            open={open}
            transition={{
                type: "spring",
                bounce: 0.1,
                duration: 0.4,
            }}
        >
            <MorphingDialogTrigger className={cn("inline-flex", variant === "default" && "w-full")}>
                <LoginTriggerButton variant={variant} />
            </MorphingDialogTrigger>
            <MorphingDialogContainer>
                <MorphingDialogContent className="relative w-[calc(100vw-2rem)] max-w-md rounded-xl shadow-2xl backdrop-blur-sm">
                    <MorphingDialogClose className="absolute top-3 right-3 z-10 flex h-8 w-8 items-center justify-center rounded-lg border border-border bg-secondary/50 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground">
                        <X className="h-4 w-4" />
                    </MorphingDialogClose>
                    <LoginContent onSuccess={() => setOpen(false)} />
                </MorphingDialogContent>
            </MorphingDialogContainer>
        </MorphingDialog>
    );
}
