"use client";

import type React from "react";
import { memo } from "react";

interface ProfileItemProps {
    icon?: React.ElementType;
    label: string;
    value: string;
}

export const ProfileItem = memo(function ProfileItem({ icon: Icon, label, value }: ProfileItemProps) {
    return (
        <div className="rounded-lg border border-border/50 bg-secondary/20 p-3">
            <div className="flex items-center gap-2 text-muted-foreground text-xs">
                {Icon && <Icon className="h-3 w-3" />}
                {label}
            </div>
            <div className="mt-1 font-medium text-foreground text-sm">{value}</div>
        </div>
    );
});
