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
        case "pack":
            return (
                <svg aria-hidden="true" viewBox="0 0 24 24" className={className} {...p}>
                    <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
                    <path d="M3.3 7 12 12l8.7-5" />
                    <path d="M12 22V12" />
                </svg>
            );
        default:
            return null;
    }
}
