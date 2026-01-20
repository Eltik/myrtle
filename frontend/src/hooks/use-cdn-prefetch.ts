"use client";

import { useCallback, useEffect, useRef } from "react";

const prefetchedUrls = new Set<string>();
const preloadedUrls = new Set<string>();

export function useCDNPrefetch() {
    const prefetch = useCallback((urls: string | string[], priority: "low" | "high" = "low") => {
        const urlArray = Array.isArray(urls) ? urls : [urls];
        const urlSet = priority === "high" ? preloadedUrls : prefetchedUrls;

        for (const url of urlArray) {
            if (urlSet.has(url)) continue;
            urlSet.add(url);

            const link = document.createElement("link");
            link.rel = priority === "high" ? "preload" : "prefetch";
            link.as = "image";
            link.href = url;
            if (priority === "high") {
                link.setAttribute("fetchpriority", "high");
            }
            document.head.appendChild(link);
        }
    }, []);

    return { prefetch };
}

// Hook to prefetch operator images when component mounts
export function usePrefetchOperatorImages(operatorIds: string[], delay = 1000) {
    const { prefetch } = useCDNPrefetch();
    const hasPrefetched = useRef(false);

    useEffect(() => {
        if (hasPrefetched.current || operatorIds.length === 0) return;
        hasPrefetched.current = true;

        const timeoutId = setTimeout(() => {
            // Prefetch first 20 operator portraits
            const urls = operatorIds.slice(0, 20).map((id) => `/api/cdn/portrait/${id}`);
            prefetch(urls);
        }, delay);

        return () => clearTimeout(timeoutId);
    }, [operatorIds, prefetch, delay]);
}
