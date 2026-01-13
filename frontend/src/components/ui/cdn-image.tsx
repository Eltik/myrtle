"use client";

import Image, { type ImageProps } from "next/image";
import { memo, useCallback, useState } from "react";
import { cn } from "~/lib/utils";

// Tiny 8x8 gray blur placeholder
const BLUR_PLACEHOLDER = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAgAAAAICAYAAADED76LAAAACXBIWXMAAAsTAAALEwEAmpwYAAAAIGNIUk0AAHolAACAgwAA+f8AAIDpAAB1MAAA6mAAADqYAAAXb5JfxUYAAAA0SURBVHjaYvj//z8DMjh06BAjAwMDIzIbpICJAR3g1UVQCDmEzABRBwMKwGsRPp2MxBgAAGsCD/+dffZmAAAAAElFTkSuQmCC";

interface CDNImageProps extends Omit<ImageProps, "placeholder" | "blurDataURL"> {
    showBlur?: boolean;
    fadeIn?: boolean;
    fallbackSrc?: string;
}

export const CDNImage = memo(function CDNImage({ showBlur = true, fadeIn = true, fallbackSrc = "/placeholder.svg", className, alt, src, onError, onLoad, ...props }: CDNImageProps) {
    const [hasError, setHasError] = useState(false);
    const [isLoaded, setIsLoaded] = useState(false);

    const handleError = useCallback(
        (e: React.SyntheticEvent<HTMLImageElement, Event>) => {
            setHasError(true);
            onError?.(e);
        },
        [onError],
    );

    const handleLoad = useCallback(
        (e: React.SyntheticEvent<HTMLImageElement, Event>) => {
            setIsLoaded(true);
            onLoad?.(e);
        },
        [onLoad],
    );

    return (
        <Image
            {...props}
            alt={alt}
            blurDataURL={showBlur ? BLUR_PLACEHOLDER : undefined}
            className={cn(fadeIn && "transition-opacity duration-300", fadeIn && !isLoaded && "opacity-0", fadeIn && isLoaded && "opacity-100", className)}
            onError={handleError}
            onLoad={handleLoad}
            placeholder={showBlur ? "blur" : "empty"}
            src={hasError ? fallbackSrc : src}
            unoptimized // CDN assets are already optimized PNGs
        />
    );
});
