"use client";

import { User } from "lucide-react";

interface LoginTriggerButtonProps {
    variant?: "default" | "header";
}

export function LoginTriggerButton({ variant = "default" }: LoginTriggerButtonProps) {
    if (variant === "header") {
        return (
            <div className="flex h-8 items-center gap-2 rounded-md border border-border bg-transparent px-3 text-foreground text-sm transition-colors hover:bg-secondary">
                <User className="h-3.5 w-3.5" />
                <span className="font-medium">Login</span>
            </div>
        );
    }

    return (
        <div className="flex h-10 w-full items-center justify-center gap-2 rounded-md border border-border bg-transparent text-foreground transition-colors hover:bg-secondary">
            <User className="h-4 w-4" />
            <span className="font-medium text-sm">Login</span>
        </div>
    );
}
