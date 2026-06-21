import { Link, useRouterState } from "@tanstack/react-router";
import { ChevronDown } from "lucide-react";
import { useRef, useState } from "react";
import { Kbd } from "#/components/ui/kbd";
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from "#/components/ui/menu";
import { useIsMac } from "#/hooks/use-is-mac";
import { ToolIcon } from "#/lib/registry/ToolIcon";
import { modKey, type ToolIconName } from "#/lib/registry/tools";
import { cn } from "#/lib/utils";
import styles from "./MainNav.module.css";

export interface INavSection {
    title: string;
    items: INavItem[];
}

export interface INavItem {
    href: string;
    label: string;
    desc?: string;
    icon?: ToolIconName;
    items?: INavItem[];
    sections?: INavSection[];
}

interface IMainNavProps extends React.ComponentProps<"nav"> {
    items: INavItem[];
    onOpenCommand?: () => void;
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

    const sections: INavSection[] = item.sections ?? (item.items ? [{ title: item.label, items: item.items }] : []);
    const showSectionTitles = (item.sections?.length ?? 0) > 1;
    const hasIcons = sections.some((section) => section.items.some((t) => t.icon));

    const renderLink = (t: INavItem) => (
        <Link key={t.href} to={t.href} className="flex items-center gap-3 rounded-lg px-3 py-2.5 no-underline transition-colors hover:bg-accent" onClick={() => setOpen(false)}>
            {hasIcons && <span className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-white/5 bg-muted/70 text-primary">{t.icon && <ToolIcon name={t.icon} />}</span>}
            <span className="flex min-w-0 flex-1 flex-col gap-0.75">
                <span className="font-medium font-sans text-[13px] text-foreground leading-none tracking-tight">{t.label}</span>
                {t.desc && <span className="font-sans text-[11.5px] text-muted-foreground leading-snug">{t.desc}</span>}
            </span>
        </Link>
    );

    return (
        // biome-ignore lint/a11y/noStaticElementInteractions: hover timers supplement the already-accessible DropdownMenu
        <div role="presentation" onMouseEnter={openNow} onMouseLeave={closeSoon}>
            <DropdownMenu open={open} onOpenChange={setOpen} modal={false}>
                <DropdownMenuTrigger className={cn(styles.navLink, isActive && styles.isActive)} aria-expanded={open}>
                    {item.label}
                    <ChevronDown className={cn(styles.chev, open && "opacity-90")} />
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" alignOffset={-8} sideOffset={10} className={cn("p-2", hasIcons ? "min-w-95" : "min-w-52")} onMouseEnter={openNow} onMouseLeave={closeSoon}>
                    {showSectionTitles ? (
                        sections.map((section, i) => (
                            <div key={section.title} className={cn(i > 0 && "mt-1.5 border-white/5 border-t pt-1.5")}>
                                <div className="px-3 pt-2 pb-1.5 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-widest">{section.title}</div>
                                <div className="flex flex-col gap-px">{section.items.map(renderLink)}</div>
                            </div>
                        ))
                    ) : (
                        <>
                            <div className="px-3 pt-2 pb-1.5 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-widest">{item.label}</div>
                            <div className="flex flex-col gap-px">{sections[0]?.items.map(renderLink)}</div>
                        </>
                    )}
                    {hasIcons && (
                        <div className="mt-1.5 flex items-center justify-between gap-2 border-white/5 border-t px-3 pt-2.5 pb-2 font-sans text-[11.5px] text-muted-foreground leading-none">
                            <span className="inline-flex items-center gap-1.5 font-medium font-mono text-[11.5px]">
                                Press <Kbd>{modKey(isMac)}</Kbd>
                                <Kbd>K</Kbd> for all commands
                            </span>
                            <button
                                type="button"
                                className="cursor-pointer border-none bg-transparent p-0 font-medium font-sans text-[11.5px] text-primary leading-none transition-colors hover:text-[color-mix(in_oklab,var(--primary),white_40%)]"
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
                const selfActive = pathname === item.href || (item.href !== "/" && pathname.startsWith(item.href));
                const childActive = item.items?.some((c) => pathname === c.href || (c.href !== "/" && pathname.startsWith(c.href))) ?? false;
                const isActive = selfActive || childActive;

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
