"use client";

import { Box } from "lucide-react";
import Image from "next/image";
import { useCallback, useState } from "react";

interface ItemIconProps {
    src: string;
    alt: string;
}

export function ItemIcon({ src, alt }: ItemIconProps) {
    const [hasError, setHasError] = useState(false);

    const handleError = useCallback(() => {
        setHasError(true);
    }, []);

    if (hasError) {
        return (
            <div className="flex h-9 w-9 items-center justify-center text-muted-foreground/50">
                <Box className="h-5 w-5" />
            </div>
        );
    }

    return <Image alt={alt} className="h-9 w-9 object-contain" height={36} onError={handleError} src={src} unoptimized width={36} />;
}

export function ItemIconLarge({ src, alt }: ItemIconProps) {
    const [hasError, setHasError] = useState(false);

    const handleError = useCallback(() => {
        setHasError(true);
    }, []);

    if (hasError) {
        return (
            <div className="flex h-20 w-20 items-center justify-center text-muted-foreground/50">
                <Box className="h-10 w-10" />
            </div>
        );
    }

    return <Image alt={alt} className="h-20 w-20 object-contain" height={80} onError={handleError} src={src} unoptimized width={80} />;
}
