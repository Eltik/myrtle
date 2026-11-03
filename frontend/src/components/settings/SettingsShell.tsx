import { type ReactNode, useEffect, useRef } from "react";
import { Kicker } from "#/components/ui/kicker";
import { Separator } from "#/components/ui/separator";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./SettingsShell.messages";

export type SettingsSectionId = "account" | "appearance" | "privacy" | "danger";

interface ISettingsNavItem {
    id: SettingsSectionId;
    label: string;
    Icon: React.ComponentType<{ className?: string }>;
}

interface ISettingsShellProps {
    nav: ISettingsNavItem[];
    active: SettingsSectionId;
    onChange: (id: SettingsSectionId) => void;
    children: ReactNode;
}

export function SettingsShell({ nav, active, onChange, children }: ISettingsShellProps) {
    const t: TypedT<typeof messages> = useT("settings");

    return (
        // A `<div>`: the site shell already wraps every page in the one `<main>`.
        <div className="page-gutter relative py-8 [--page-max:1100px] sm:py-14">
            <div className="mb-6 sm:mb-8">
                <h1 className="m-0 font-(--font-heading) font-bold text-[26px] text-foreground leading-[1.15] tracking-[-0.02em] sm:text-[36px]">{t("shell.title")}</h1>
                <p className="m-0 mt-1.5 max-w-[60ch] font-sans text-[14px] text-muted-foreground leading-[1.55] sm:text-[15px]">{t("shell.subtitle")}</p>
            </div>

            {/* One section (signed out: Appearance alone) needs no section nav; the sticky
                strip spent about 85px of a phone screen naming the only page there is. */}
            <div className={cn("grid grid-cols-1 items-start gap-5", nav.length > 1 && "sm:grid-cols-[232px_1fr] sm:gap-8")}>
                {nav.length > 1 ? <SettingsNav nav={nav} active={active} onChange={onChange} /> : null}
                {/* Below sm the card's 24px padding stacked on the page gutter put every row
                    about 37px in from each screen edge; 16px lets the labels breathe. */}
                <div className="min-w-0 max-sm:[&_[data-slot=card]>[data-slot^=card-]]:px-4">{children}</div>
            </div>
        </div>
    );
}

function SettingsNav({ nav, active, onChange }: { nav: ISettingsNavItem[]; active: SettingsSectionId; onChange: (id: SettingsSectionId) => void }) {
    const t: TypedT<typeof messages> = useT("settings");
    const scrollerRef = useRef<HTMLDivElement>(null);
    const activeRef = useRef<HTMLButtonElement>(null);

    // biome-ignore lint/correctness/useExhaustiveDependencies: re-scroll when the active id changes; refs settle synchronously
    useEffect(() => {
        const scroller = scrollerRef.current;
        const node = activeRef.current;
        if (!scroller || !node) return;
        const sNode = node.offsetLeft;
        const sLeft = scroller.scrollLeft;
        const sRight = sLeft + scroller.clientWidth;
        if (sNode < sLeft + 16 || sNode + node.offsetWidth > sRight - 16) {
            scroller.scrollTo({ left: sNode - 16, behavior: "smooth" });
        }
    }, [active]);

    const mainItems = nav.filter((item) => item.id !== "danger");
    const dangerItems = nav.filter((item) => item.id === "danger");

    return (
        <nav aria-label={t("shell.sectionsNav")} className={cn("page-bleed sm:mx-0", "sticky top-(--site-header-height) z-20 border-border border-b bg-background", "sm:top-20 sm:z-0 sm:self-start sm:rounded-2xl sm:border sm:border-sidebar-border sm:bg-sidebar/60 sm:p-2")}>
            <div ref={scrollerRef} className="max-sm:scroll-shadows-x flex gap-1 overflow-x-auto px-3 py-2 [scrollbar-width:none] sm:flex-col sm:gap-0.5 sm:overflow-visible sm:p-0 [&::-webkit-scrollbar]:hidden">
                {mainItems.map((item) => (
                    <SettingsNavButton key={item.id} item={item} isActive={active === item.id} activeRef={active === item.id ? activeRef : undefined} onSelect={onChange} />
                ))}
                {dangerItems.length > 0 ? (
                    <>
                        <Separator className="mx-1 hidden sm:my-1.5 sm:block sm:bg-sidebar-border" />
                        {dangerItems.map((item) => (
                            <SettingsNavButton key={item.id} item={item} isActive={active === item.id} activeRef={active === item.id ? activeRef : undefined} onSelect={onChange} tone="danger" />
                        ))}
                    </>
                ) : null}
            </div>
        </nav>
    );
}

function SettingsNavButton({ item, isActive, activeRef, onSelect, tone = "default" }: { item: ISettingsNavItem; isActive: boolean; activeRef?: React.RefObject<HTMLButtonElement | null>; onSelect: (id: SettingsSectionId) => void; tone?: "default" | "danger" }) {
    const { id, label, Icon } = item;
    return (
        <button
            ref={activeRef}
            type="button"
            onClick={() => onSelect(id)}
            aria-current={isActive ? "page" : undefined}
            data-active={isActive ? "true" : undefined}
            className={cn(
                "group inline-flex shrink-0 cursor-pointer items-center gap-2.5 whitespace-nowrap rounded-lg border border-transparent px-3 py-2 text-left font-sans text-[13.5px] outline-none transition-colors focus-visible:ring-2 focus-visible:ring-sidebar-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background",
                // 44px on the phone strip, where these are the only way between sections.
                "min-h-11 touch-manipulation sm:min-h-9",
                "sm:w-full sm:px-2.5 sm:py-2 sm:text-[14px]",
                tone === "danger" ? (isActive ? "bg-destructive/12 font-medium text-destructive" : "text-muted-foreground hover:bg-destructive/10 hover:text-destructive") : isActive ? "bg-accent font-medium text-foreground" : "text-muted-foreground hover:bg-sidebar-accent/60 hover:text-sidebar-accent-foreground",
            )}
        >
            <span aria-hidden className={cn("pointer-events-none absolute top-1.5 bottom-1.5 left-0 hidden w-0.75 rounded-r-full transition-opacity sm:block", isActive ? "opacity-100" : "opacity-0", tone === "danger" ? "bg-destructive" : "bg-primary")} />
            <Icon className={cn("size-4 shrink-0", isActive ? "opacity-100" : "opacity-80")} />
            <span className="min-w-0">{label}</span>
        </button>
    );
}

export function SettingRow({ title, description, control, layout = "auto" }: { title: ReactNode; description?: ReactNode; control: ReactNode; layout?: "auto" | "inline" }) {
    const stack = layout === "auto";
    return (
        <div className={cn("border-border border-t py-3.5 first:border-t-0", stack ? "flex flex-col gap-2.5 sm:grid sm:grid-cols-[1fr_auto] sm:items-center sm:gap-x-4 sm:gap-y-1" : "grid grid-cols-[1fr_auto] items-center gap-x-4 gap-y-1")}>
            <div className="flex min-w-0 flex-col gap-0.5">
                <span className="font-medium font-sans text-[14px] text-foreground leading-[1.35]">{title}</span>
                {description ? <span className="font-sans text-[13px] text-muted-foreground leading-normal">{description}</span> : null}
            </div>
            <div className={cn("flex shrink-0 items-center", stack ? "justify-start sm:justify-end" : "justify-end")}>{control}</div>
        </div>
    );
}

export function SectionLabel({ children }: { children: ReactNode }) {
    return <Kicker className="mb-2.5">{children}</Kicker>;
}

export function Mono({ children }: { children: ReactNode }) {
    return <code className="font-mono text-[12px] text-foreground/85">{children}</code>;
}
