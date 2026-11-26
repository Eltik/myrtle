import { Link, useRouterState } from "@tanstack/react-router";
import { ChevronDown } from "lucide-react";
import { Fragment, useState } from "react";
import { Kbd } from "#/components/ui/kbd";
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "#/components/ui/menu";
import { useIsMac } from "#/hooks/use-is-mac";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { ToolIcon } from "#/lib/registry/ToolIcon";
import { modKey, type ToolIconName } from "#/lib/registry/tools";
import { cn } from "#/lib/utils";
import type { messages } from "./MainNav.messages";
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

/** The mono caps heading every dropdown section has always worn, now on the menu's own group label. */
// `.kicker` voice; the size and weight are restated because MenuGroupLabel's `text-xs font-medium` utilities outrank the component layer.
const SECTION_LABEL = "kicker px-3 pt-2 pb-1.5 font-semibold text-[10.5px] leading-none";

interface IMainNavProps extends React.ComponentProps<"nav"> {
    items: INavItem[];
    onOpenCommand?: () => void;
    /**
     * The nav does not fit the header row. It stays laid out, out of flow and
     * invisible, so the header can keep measuring its natural width.
     */
    collapsed?: boolean;
}

interface IHoverDropdownProps {
    item: INavItem;
    isActive: boolean;
    open: boolean;
    setOpen: (open: boolean) => void;
    onOpenCommand?: () => void;
}

function HoverDropdown({ item, isActive, open, setOpen, onOpenCommand }: IHoverDropdownProps) {
    const isMac = useIsMac();
    const t: TypedT<typeof messages> = useT("nav");

    const sections: INavSection[] = item.sections ?? (item.items ? [{ title: item.label, items: item.items }] : []);
    const showSectionTitles = (item.sections?.length ?? 0) > 1;
    const hasIcons = sections.some((section) => section.items.some((t) => t.icon));

    // A menu item rendered as a Link, not a bare Link: Base UI only roves
    // arrow-key focus across registered items, so plain anchors left ArrowDown
    // dead (focus stayed on the first row), and a click on an item closes the
    // menu without an onClick of our own.
    const renderLink = (t: INavItem) => (
        <DropdownMenuItem key={t.href} className="cursor-pointer gap-3 rounded-lg px-3 py-2.5 no-underline transition-colors" render={<Link to={t.href} />}>
            {hasIcons && <span className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-white/5 bg-muted/70 text-primary">{t.icon && <ToolIcon name={t.icon} />}</span>}
            <span className="flex min-w-0 flex-1 flex-col gap-0.75">
                <span className="font-medium font-sans text-[13px] text-foreground leading-none tracking-tight">{t.label}</span>
                {t.desc && <span className="font-sans text-[11.5px] text-muted-foreground leading-snug">{t.desc}</span>}
            </span>
        </DropdownMenuItem>
    );

    return (
        // Hover and click both go through Base UI. The old hand-rolled hover timers opened the
        // menu on mouseenter, so the click that followed toggled it straight back shut and a
        // mouse click read as a dead control (audit-002 Z1); Base UI's hover keeps it open.
        <DropdownMenu open={open} onOpenChange={setOpen} modal={false}>
            <DropdownMenuTrigger className={cn(styles.navLink, isActive && styles.isActive)} openOnHover delay={0} closeDelay={140}>
                {item.label}
                <ChevronDown className={cn(styles.chev, open && "opacity-90")} />
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" alignOffset={-8} sideOffset={10} className={cn("p-2", hasIcons ? "min-w-95" : "min-w-52")}>
                {/* Each section is a menu group labelled by its heading, so a
                        screen reader announces "Game Data, group" on entering it,
                        while arrow keys still run straight through every group. */}
                {showSectionTitles ? (
                    sections.map((section, i) => (
                        <Fragment key={section.title}>
                            {i > 0 && <DropdownMenuSeparator className="mx-3 my-1.5" />}
                            <DropdownMenuGroup className="flex flex-col gap-px">
                                <DropdownMenuLabel className={SECTION_LABEL}>{section.title}</DropdownMenuLabel>
                                {section.items.map(renderLink)}
                            </DropdownMenuGroup>
                        </Fragment>
                    ))
                ) : (
                    <DropdownMenuGroup className="flex flex-col gap-px">
                        <DropdownMenuLabel className={SECTION_LABEL}>{item.label}</DropdownMenuLabel>
                        {sections[0]?.items.map(renderLink)}
                    </DropdownMenuGroup>
                )}
                {hasIcons && (
                    <div className="mt-1.5 flex items-center justify-between gap-2 border-border border-t px-3 pt-2.5 pb-2 font-sans text-[11.5px] text-muted-foreground leading-none">
                        <span className="inline-flex items-center gap-1.5 font-medium font-mono text-[11.5px]">
                            {t("mainNav.press")} <Kbd>{modKey(isMac)}</Kbd>
                            <Kbd>K</Kbd> {t("mainNav.forAllCommands")}
                        </span>
                        <button
                            type="button"
                            className="cursor-pointer border-none bg-transparent p-0 font-medium font-sans text-[11.5px] text-primary leading-none underline-offset-2 hover:underline"
                            onClick={() => {
                                setOpen(false);
                                onOpenCommand?.();
                            }}
                        >
                            {t("mainNav.openPalette")}
                        </button>
                    </div>
                )}
            </DropdownMenuContent>
        </DropdownMenu>
    );
}

export function MainNav({ items, className, onOpenCommand, collapsed = false, ...props }: IMainNavProps) {
    const router = useRouterState();
    const pathname = router.location.pathname;
    // One open menu for the whole bar. Each menu kept its own state, so a menu opened by a
    // click stayed open while the pointer moved onto the next trigger, and that trigger's
    // hover never took over (seen in the 2026-11-04 browser pass).
    const [openHref, setOpenHref] = useState<string | null>(null);

    return (
        <nav className={cn("hidden shrink-0 items-center gap-1 lg:flex", collapsed && "pointer-events-none invisible absolute top-0 left-0", className)} aria-hidden={collapsed || undefined} {...props}>
            {items.map((item) => {
                const selfActive = pathname === item.href || (item.href !== "/" && pathname.startsWith(item.href));
                const childActive = item.items?.some((c) => pathname === c.href || (c.href !== "/" && pathname.startsWith(c.href))) ?? false;
                const isActive = selfActive || childActive;

                if (item.items) {
                    return <HoverDropdown key={item.href} item={item} isActive={isActive} open={openHref === item.href} setOpen={(next) => setOpenHref((cur) => (next ? item.href : cur === item.href ? null : cur))} onOpenCommand={onOpenCommand} />;
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
