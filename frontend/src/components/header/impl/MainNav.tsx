import { Link, useRouterState } from "@tanstack/react-router";
import { Button } from "#/components/ui/button";
import { cn } from "#/lib/utils";
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from "#/components/ui/menu";
import { ChevronDown } from "lucide-react";
import { Kbd } from "#/components/ui/kbd";
import { useRef, useState } from "react";

export interface NavItem {
    href: string;
    label: string;
    desc?: string;
    icon?: string;
    kb?: string[] | null;
    items?: NavItem[];
}

interface MainNavProps extends React.ComponentProps<"nav"> {
    items: NavItem[];
    onOpenCommand?: () => void;
}

function ToolsIcon({ name }: { name: string }) {
    const p = { fill: "none", stroke: "currentColor", strokeWidth: "1.9", strokeLinecap: "round" as const, strokeLinejoin: "round" as const };
    switch (name) {
        case "chart":
            return (
                <svg viewBox="0 0 24 24" {...p}>
                    <path d="M3 3v18h18" />
                    <path d="m19 9-5 5-4-4-3 3" />
                </svg>
            );
        case "calc":
            return (
                <svg viewBox="0 0 24 24" {...p}>
                    <rect x="4" y="3" width="16" height="18" rx="2" />
                    <path d="M8 7h8M8 12h2M14 12h2M8 17h2M14 17h2" />
                </svg>
            );
        case "star":
            return (
                <svg viewBox="0 0 24 24" {...p}>
                    <path d="m12 2 3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                </svg>
            );
        case "dice":
            return (
                <svg viewBox="0 0 24 24" {...p}>
                    <rect x="3" y="3" width="18" height="18" rx="2" />
                    <path d="M8 8h.01M16 8h.01M8 16h.01M16 16h.01M12 12h.01" />
                </svg>
            );
        case "pack":
            return (
                <svg viewBox="0 0 24 24" {...p}>
                    <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
                    <path d="M3.3 7 12 12l8.7-5" />
                    <path d="M12 22V12" />
                </svg>
            );
        default:
            return null;
    }
}

function ToolsDropdown({ items, onClose, onOpenCommand }: { items: NavItem[]; onClose: () => void; onOpenCommand?: () => void }) {
    return (
        <div className="nav-menu" role="menu">
            <div className="nav-menu-label">Tools</div>
            <div className="nav-menu-list">
                {items.map((t) => (
                    <Link key={t.href} to={t.href} className="nav-menu-item" role="menuitem" onClick={onClose}>
                        <span className="nav-menu-icon">{t.icon && <ToolsIcon name={t.icon} />}</span>
                        <span className="nav-menu-text">
                            <span className="nav-menu-title">{t.label}</span>
                            {t.desc && <span className="nav-menu-desc">{t.desc}</span>}
                        </span>
                        {t.kb && (
                            <span className="nav-menu-kbd">
                                {t.kb.map((k, j) => (
                                    <Kbd key={j}>{k}</Kbd>
                                ))}
                            </span>
                        )}
                    </Link>
                ))}
            </div>
            <div className="nav-menu-foot">
                <span className="small-mono">
                    Press <Kbd>⌘</Kbd>
                    <Kbd>K</Kbd> for all commands
                </span>
                <button
                    type="button"
                    className="pill-link"
                    onClick={() => {
                        onClose();
                        onOpenCommand?.();
                    }}
                >
                    Open palette →
                </button>
            </div>
        </div>
    );
}

export function MainNav({ items, className, onOpenCommand, ...props }: MainNavProps) {
    const router = useRouterState();
    const pathname = router.location.pathname;
    const [openMenu, setOpenMenu] = useState<string | null>(null);
    const closeTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

    const openNow = (href: string) => {
        if (closeTimer.current) clearTimeout(closeTimer.current);
        setOpenMenu(href);
    };

    const closeSoon = () => {
        if (closeTimer.current) clearTimeout(closeTimer.current);
        closeTimer.current = setTimeout(() => setOpenMenu(null), 140);
    };

    return (
        <nav className={cn("hidden items-center gap-1 lg:flex", className)} {...props}>
            {items.map((item) => {
                const isActive = pathname === item.href || (item.href !== "/" && pathname.startsWith(item.href));
                const isOpen = openMenu === item.href;

                if (item.items) {
                    return (
                        <div key={item.href} className="nav-item relative" onMouseEnter={() => openNow(item.href)} onMouseLeave={closeSoon}>
                            <button type="button" className={cn("nav-link", isActive && "is-active")} aria-haspopup="menu" aria-expanded={isOpen} onClick={() => setOpenMenu(isOpen ? null : item.href)}>
                                {item.label}
                                <svg className="chev" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{ transform: isOpen ? "rotate(180deg)" : undefined, opacity: isOpen ? 0.9 : 0.6 }}>
                                    <path d="m6 9 6 6 6-6" />
                                </svg>
                            </button>
                            {isOpen && <ToolsDropdown items={item.items} onClose={() => setOpenMenu(null)} onOpenCommand={onOpenCommand} />}
                        </div>
                    );
                }

                return (
                    <Link key={item.href} to={item.href} className={cn("nav-link", isActive && "is-active")}>
                        {item.label}
                    </Link>
                );
            })}
        </nav>
    );
}
