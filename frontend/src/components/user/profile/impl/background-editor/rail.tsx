import type { ReactNode } from "react";
import { cn } from "#/lib/utils";
import { COUNT } from "./chips";

/** A rail row that picks one thing: its icon, its name and its muted count at the end. */
export function RailRow({ active, onClick, name, count, title, icon }: { active: boolean; onClick: () => void; name: string; count?: number | string | null; title?: string; icon?: ReactNode }) {
    return (
        <button
            type="button"
            aria-pressed={active}
            onClick={onClick}
            title={title}
            className={cn(
                "flex w-full cursor-pointer items-center gap-2.5 rounded-md px-2 py-1.5 text-start font-sans text-[13px] leading-tight outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring [&_svg]:size-4 [&_svg]:shrink-0 [&_svg]:opacity-70",
                active ? "bg-primary/12 font-medium text-foreground [&_svg]:opacity-100" : "text-muted-foreground hover:bg-accent hover:text-foreground",
            )}
        >
            {icon}
            <span className="min-w-0 flex-1 truncate">{name}</span>
            {count !== undefined && count !== null && <span className={COUNT}>{count}</span>}
        </button>
    );
}

/** A quiet label over a group of the rail: small, muted, no heavy heading. */
export function RailLabel({ children }: { children: ReactNode }) {
    return <p className="m-0 mb-1.5 px-2 font-medium font-sans text-[11px] text-muted-foreground/80">{children}</p>;
}
