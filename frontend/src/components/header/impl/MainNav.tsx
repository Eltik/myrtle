import { Link, useRouterState } from "@tanstack/react-router";
import { ChevronDown } from "lucide-react";
import { useRef, useState } from "react";
import { Kbd } from "#/components/ui/kbd";
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from "#/components/ui/menu";
import { useIsMac } from "#/hooks/use-is-mac";
import { modKey } from "#/lib/registry/tools";
import { cn } from "#/lib/utils";
import styles from "./MainNav.module.css";

export interface INavItem {
    href: string;
    label: string;
    desc?: string;
    icon?: string;
    kb?: string[] | null;
    items?: INavItem[];
}

interface IMainNavProps extends React.ComponentProps<"nav"> {
    items: INavItem[];
    onOpenCommand?: () => void;
}

function ToolsIcon({ name }: { name: string }) {
    const p = { fill: "none", stroke: "currentColor", strokeWidth: "1.9", strokeLinecap: "round" as const, strokeLinejoin: "round" as const };
    switch (name) {
        case "chart":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <path d="M3 3v18h18" />
                    <path d="m19 9-5 5-4-4-3 3" />
                </svg>
            );
        case "calc":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <rect x="4" y="3" width="16" height="18" rx="2" />
                    <path d="M8 7h8M8 12h2M14 12h2M8 17h2M14 17h2" />
                </svg>
            );
        case "star":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <path d="m12 2 3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                </svg>
            );
        case "dice":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <rect x="3" y="3" width="18" height="18" rx="2" />
                    <path d="M8 8h.01M16 8h.01M8 16h.01M16 16h.01M12 12h.01" />
                </svg>
            );
        case "pack":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
                    <path d="M3.3 7 12 12l8.7-5" />
                    <path d="M12 22V12" />
                </svg>
            );
        case "search":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <circle cx="11" cy="11" r="7" />
                    <path d="m20 20-3.5-3.5" />
                </svg>
            );
        case "trophy":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <path d="M6 4h12v3a6 6 0 1 1-12 0V4z" />
                    <path d="M6 6H4a2 2 0 0 0 2 4M18 6h2a2 2 0 0 1-2 4" />
                    <path d="M9 17h6M10 17v3h4v-3M8 20h8" />
                </svg>
            );
        case "users":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
                    <circle cx="9" cy="7" r="4" />
                    <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
                    <path d="M16 3.13a4 4 0 0 1 0 7.75" />
                </svg>
            );
        case "user":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                    <circle cx="12" cy="7" r="4" />
                </svg>
            );
        case "history":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <path d="M3 12a9 9 0 1 0 3-6.7" />
                    <path d="M3 4v5h5" />
                    <path d="M12 7v5l3 2" />
                </svg>
            );
        default:
            return null;
    }
}

function HoverDropdown({ item, isActive, onOpenCommand }: { item: INavItem; isActive: boolean; onOpenCommand?: () => void }) {
    const [open, setOpen] = useState(false);
    const closeTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

    const openNow = () => {
        if (closeTimer.current) clearTimeout(closeTimer.current);
        setOpen(true);
    };
    const closeSoon = () => {
        if (closeTimer.current) clearTimeout(closeTimer.current);
        closeTimer.current = setTimeout(() => setOpen(false), 140);
    };

    const isMac = useIsMac();
    const hasIcons = item.items?.some((t) => t.icon);

    return (
        // biome-ignore lint/a11y/noStaticElementInteractions: hover timers supplement the already-accessible DropdownMenu
        <div role="presentation" onMouseEnter={openNow} onMouseLeave={closeSoon}>
            <DropdownMenu open={open} onOpenChange={setOpen} modal={false}>
                <DropdownMenuTrigger className={cn(styles.navLink, isActive && styles.isActive)} aria-expanded={open}>
                    {item.label}
                    <ChevronDown className={cn(styles.chev, open && "opacity-90")} />
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" alignOffset={-8} sideOffset={10} className={cn("p-2", hasIcons ? "min-w-95" : "min-w-52")} onMouseEnter={openNow} onMouseLeave={closeSoon}>
                    <div className="px-3 pt-2 pb-1.5 font-mono text-[10.5px] font-medium uppercase leading-none tracking-widest text-muted-foreground">{item.label}</div>
                    <div className="flex flex-col gap-px">
                        {item.items?.map((t) => (
                            <Link key={t.href} to={t.href} className="flex items-center gap-3 rounded-lg px-3 py-2.5 no-underline transition-colors hover:bg-accent" onClick={() => setOpen(false)}>
                                {hasIcons && <span className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-white/5 bg-muted/70 text-primary">{t.icon && <ToolsIcon name={t.icon} />}</span>}
                                <span className="flex min-w-0 flex-1 flex-col gap-0.75">
                                    <span className="font-sans text-[13px] font-medium leading-none tracking-tight text-foreground">{t.label}</span>
                                    {t.desc && <span className="font-sans text-[11.5px] leading-snug text-muted-foreground">{t.desc}</span>}
                                </span>
                                {t.kb && (
                                    <span className="inline-flex shrink-0 gap-0.5">
                                        {t.kb.map((k) => (
                                            <Kbd key={k}>{k}</Kbd>
                                        ))}
                                    </span>
                                )}
                            </Link>
                        ))}
                    </div>
                    {hasIcons && (
                        <div className="mt-1.5 flex items-center justify-between gap-2 border-t border-white/5 px-3 pt-2.5 pb-2 font-sans text-[11.5px] leading-none text-muted-foreground">
                            <span className="inline-flex items-center gap-1.5 font-mono text-[11.5px] font-medium">
                                Press <Kbd>{modKey(isMac)}</Kbd>
                                <Kbd>K</Kbd> for all commands
                            </span>
                            <button
                                type="button"
                                className="cursor-pointer border-none bg-transparent p-0 font-sans text-[11.5px] font-medium leading-none text-primary transition-colors hover:text-[oklch(0.85_0.12_25)]"
                                onClick={() => {
                                    setOpen(false);
                                    onOpenCommand?.();
                                }}
                            >
                                Open palette →
                            </button>
                        </div>
                    )}
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    );
}

export function MainNav({ items, className, onOpenCommand, ...props }: IMainNavProps) {
    const router = useRouterState();
    const pathname = router.location.pathname;

    return (
        <nav className={cn("hidden items-center gap-1 lg:flex", className)} {...props}>
            {items.map((item) => {
                const isActive = pathname === item.href || (item.href !== "/" && pathname.startsWith(item.href));

                if (item.items) {
                    return <HoverDropdown key={item.href} item={item} isActive={isActive} onOpenCommand={onOpenCommand} />;
                }

                return (
                    <Link key={item.href} to={item.href} className={cn(styles.navLink, isActive && styles.isActive)}>
                        {item.label}
                    </Link>
                );
            })}
        </nav>
    );
}
