"use client";

import { Crown, Users } from "lucide-react";
import { Badge } from "~/components/ui/shadcn/badge";
import { cn } from "~/lib/utils";
import type { TierListType } from "~/types/api/impl/tier-list";

interface TierListTypeBadgeProps {
    type?: TierListType | null;
    className?: string;
    showIcon?: boolean;
}

const TYPE_STYLES = {
    official: {
        className: "bg-amber-500/10 text-amber-500 border-amber-500/20",
        icon: Crown,
        label: "Official",
    },
    community: {
        className: "bg-blue-500/10 text-blue-500 border-blue-500/20",
        icon: Users,
        label: "Community",
    },
} as const;

export function TierListTypeBadge({ type, className, showIcon = true }: TierListTypeBadgeProps) {
    // Default to "official" if type is undefined or null
    const effectiveType = type && type in TYPE_STYLES ? type : "official";
    const config = TYPE_STYLES[effectiveType];
    const Icon = config.icon;

    return (
        <Badge className={cn(config.className, "gap-1 border font-medium", className)} variant="outline">
            {showIcon && <Icon className="h-3 w-3" />}
            {config.label}
        </Badge>
    );
}
