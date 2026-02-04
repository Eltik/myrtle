"use client";

import Image from "next/image";
import { cn } from "~/lib/utils";

interface MaterialItemProps {
    id: string;
    count: number;
    image?: string | null;
    size?: "sm" | "md";
}

export function MaterialItem({ id, count, image, size = "md" }: MaterialItemProps) {
    const sizeClass = size === "sm" ? "h-10 w-10" : "h-12 w-12";
    // Use backend-provided image path if available, otherwise fallback to default path
    const imageSrc = image ? `/api/cdn${image}` : `/api/cdn/upk/arts/items/icons/${id}.png`;

    return (
        <div className="flex flex-col items-center gap-1">
            <div className={cn("relative rounded-lg border border-border/50 bg-secondary/30 p-1", sizeClass)}>
                <Image alt={id} className="object-contain" fill src={imageSrc} />
            </div>
            <span className="font-mono text-foreground text-xs">x{count}</span>
        </div>
    );
}
