import type React from "react";
import type { ToolIconName } from "#/lib/registry/tools";

export function ToolIcon({ name, className }: { name: ToolIconName; className?: string }): React.ReactElement | null {
    const p = {
        fill: "none",
        stroke: "currentColor",
        strokeWidth: 2,
        strokeLinecap: "round" as const,
        strokeLinejoin: "round" as const,
    };
    switch (name) {
        case "calc":
            return (
                <svg aria-hidden="true" viewBox="0 0 24 24" className={className} {...p}>
                    <rect x="4" y="3" width="16" height="18" rx="2" />
                    <path d="M8 7h8M8 12h2M14 12h2M8 17h2M14 17h2" />
                </svg>
            );
        case "chart":
            return (
                <svg aria-hidden="true" viewBox="0 0 24 24" className={className} {...p}>
                    <path d="M3 3v18h18" />
                    <path d="m19 9-5 5-4-4-3 3" />
                </svg>
            );
        case "heart":
            return (
                <svg aria-hidden="true" viewBox="0 0 24 24" className={className} {...p}>
                    <path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.29 1.51 4.04 3 5.5l7 7Z" />
                    <path d="M3.5 12h4l2-3 3 5 2-3h4" />
                </svg>
            );
        case "star":
            return (
                <svg aria-hidden="true" viewBox="0 0 24 24" className={className} {...p}>
                    <path d="m12 2 3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                </svg>
            );
        case "dice":
            return (
                <svg aria-hidden="true" viewBox="0 0 24 24" className={className} {...p}>
                    <rect x="3" y="3" width="18" height="18" rx="2" />
                    <path d="M8 8h.01M16 8h.01M8 16h.01M16 16h.01M12 12h.01" />
                </svg>
            );
        case "cake":
            return (
                <svg aria-hidden="true" viewBox="0 0 24 24" className={className} {...p}>
                    <path d="M20 21v-8a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8" />
                    <path d="M4 16s.5-1 2-1 2.5 2 4 2 2.5-2 4-2 2.5 2 4 2 2-1 2-1" />
                    <path d="M2 21h20" />
                    <path d="M7 8v2M12 8v2M17 8v2" />
                    <path d="M7 4h.01M12 4h.01M17 4h.01" />
                </svg>
            );
        case "pack":
            return (
                <svg aria-hidden="true" viewBox="0 0 24 24" className={className} {...p}>
                    <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
                    <path d="M3.3 7 12 12l8.7-5" />
                    <path d="M12 22V12" />
                </svg>
            );
        case "tiers":
            return (
                <svg aria-hidden="true" viewBox="0 0 24 24" className={className} {...p}>
                    <rect x="3" y="4" width="18" height="4" rx="1" />
                    <rect x="3" y="10" width="18" height="4" rx="1" />
                    <rect x="3" y="16" width="18" height="4" rx="1" />
                    <path d="M8 6h0M8 12h0M8 18h0" />
                </svg>
            );
        case "search":
            return (
                <svg aria-hidden="true" viewBox="0 0 24 24" className={className} {...p}>
                    <circle cx="11" cy="11" r="7" />
                    <path d="m20 20-3.5-3.5" />
                </svg>
            );
        case "trophy":
            return (
                <svg aria-hidden="true" viewBox="0 0 24 24" className={className} {...p}>
                    <path d="M6 4h12v3a6 6 0 1 1-12 0V4z" />
                    <path d="M6 6H4a2 2 0 0 0 2 4M18 6h2a2 2 0 0 1-2 4" />
                    <path d="M9 17h6M10 17v3h4v-3M8 20h8" />
                </svg>
            );
        case "users":
            return (
                <svg aria-hidden="true" viewBox="0 0 24 24" className={className} {...p}>
                    <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
                    <circle cx="9" cy="7" r="4" />
                    <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
                    <path d="M16 3.13a4 4 0 0 1 0 7.75" />
                </svg>
            );
        case "history":
            return (
                <svg aria-hidden="true" viewBox="0 0 24 24" className={className} {...p}>
                    <path d="M3 12a9 9 0 1 0 3-6.7" />
                    <path d="M3 4v5h5" />
                    <path d="M12 7v5l3 2" />
                </svg>
            );
        default:
            return null;
    }
}
