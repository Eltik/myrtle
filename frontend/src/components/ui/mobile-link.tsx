"use client";

import Link, { type LinkProps } from "next/link";
import { type AnchorHTMLAttributes, forwardRef } from "react";
import { useIsMobile } from "~/hooks/use-mobile";

type MobileLinkProps = Omit<AnchorHTMLAttributes<HTMLAnchorElement>, keyof LinkProps> &
    LinkProps & {
        children?: React.ReactNode;
    };

/**
 * Mobile-optimized Link component that disables prefetching on mobile devices.
 * This reduces network requests and improves performance on mobile networks.
 *
 * On desktop, prefetching works as normal.
 * On mobile (touch devices or narrow viewports), prefetching is disabled.
 */
export const MobileLink = forwardRef<HTMLAnchorElement, MobileLinkProps>(function MobileLink({ prefetch, ...props }, ref) {
    const isMobile = useIsMobile();

    // On mobile, disable prefetching to reduce network requests
    // On desktop, use the provided prefetch value or default behavior
    const shouldPrefetch = isMobile ? false : prefetch;

    return <Link prefetch={shouldPrefetch} ref={ref} {...props} />;
});
