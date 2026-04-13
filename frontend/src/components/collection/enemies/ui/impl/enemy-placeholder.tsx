import { cn } from "~/lib/utils";

interface EnemyPlaceholderProps {
    className?: string;
}

export function EnemyPlaceholder({ className }: EnemyPlaceholderProps) {
    return (
        <svg className={cn("text-muted-foreground/40", className)} fill="none" role="img" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
            <title>Enemy placeholder</title>
            <rect fill="currentColor" height="100" opacity="0.08" rx="8" width="100" />
            {/* Enemy silhouette */}
            <circle cx="50" cy="36" fill="currentColor" opacity="0.3" r="14" />
            <path d="M30 78c0-11.046 8.954-20 20-20s20 8.954 20 20" fill="currentColor" opacity="0.2" />
            {/* Crosshair marks */}
            <line opacity="0.15" stroke="currentColor" strokeWidth="1.5" x1="50" x2="50" y1="12" y2="18" />
            <line opacity="0.15" stroke="currentColor" strokeWidth="1.5" x1="50" x2="50" y1="54" y2="60" />
            <line opacity="0.15" stroke="currentColor" strokeWidth="1.5" x1="26" x2="32" y1="36" y2="36" />
            <line opacity="0.15" stroke="currentColor" strokeWidth="1.5" x1="68" x2="74" y1="36" y2="36" />
        </svg>
    );
}
