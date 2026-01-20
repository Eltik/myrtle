"use client";

import Image, { type ImageProps } from "next/image";
import { memo, useCallback, useState } from "react";
import { cn } from "~/lib/utils";

interface ImageWithSkeletonProps extends Omit<ImageProps, "onLoad" | "onError"> {
    skeletonClassName?: string;
    containerClassName?: string;
}

export const ImageWithSkeleton = memo(function ImageWithSkeleton({ className, skeletonClassName, containerClassName, alt, ...props }: ImageWithSkeletonProps) {
    const [isLoaded, setIsLoaded] = useState(false);
    const [hasError, setHasError] = useState(false);

    const handleLoad = useCallback(() => {
        setIsLoaded(true);
    }, []);

    const handleError = useCallback(() => {
        setHasError(true);
    }, []);

    const showSkeleton = !isLoaded || hasError;

    return (
        <div className={cn("relative overflow-hidden", containerClassName)}>
            {/* Skeleton placeholder with shimmer effect */}
            {showSkeleton && (
                <div aria-hidden="true" className={cn("absolute inset-0 bg-muted", skeletonClassName)}>
                    {/* Shimmer overlay - only animate while loading, not on error */}
                    {!hasError && <div className="skeleton-shimmer absolute inset-0" />}
                </div>
            )}

            {/* Actual image - hide completely on error */}
            {!hasError && <Image {...props} alt={alt} className={cn("transition-opacity duration-300", isLoaded ? "opacity-100" : "opacity-0", className)} onError={handleError} onLoad={handleLoad} />}
        </div>
    );
});
