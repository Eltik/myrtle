"use client";

import Image from "next/image";
import { cn, formatSubProfession } from "~/lib/utils";

interface SubClassIconProps {
    subProfessionId: string;
    size?: number;
    className?: string;
}

export function SubClassIcon({ subProfessionId, size = 24, className }: SubClassIconProps) {
    const displayName = formatSubProfession(subProfessionId);

    return <Image alt={displayName} className={cn("icon-theme-aware", className)} height={size} src={`/api/cdn/upk/spritepack/ui_sub_profession_icon_hub_h2_0/sub_${subProfessionId}_icon.png`} width={size} />;
}
