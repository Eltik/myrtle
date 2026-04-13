"use client";

import { useState } from "react";
import { cn } from "~/lib/utils";
import { Dialog, DialogContent, DialogTrigger } from "../../ui/shadcn/dialog";
import { LoginContent } from "./impl/login-content";
import { LoginTriggerButton } from "./impl/login-trigger-button";
import type { LoginProps } from "./impl/types";

export function Login({ variant = "default" }: LoginProps) {
    const [open, setOpen] = useState(false);

    return (
        <Dialog onOpenChange={setOpen} open={open}>
            <DialogTrigger className={cn("inline-flex", variant === "default" && "w-full")}>
                <LoginTriggerButton variant={variant} />
            </DialogTrigger>
            <DialogContent className="border-none bg-transparent p-0 shadow-none" showCloseButton={false}>
                <LoginContent onSuccess={() => setOpen(false)} />
            </DialogContent>
        </Dialog>
    );
}
