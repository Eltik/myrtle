import { type ReactNode, useEffect, useRef } from "react";
import { Kicker } from "#/components/ui/kicker";
import { cn } from "#/lib/utils";

export type SettingsSectionId = "profile" | "appearance" | "privacy" | "data" | "danger";

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
    return (
        <main className="relative mx-auto w-[min(1100px,calc(100%-2rem))] py-8 sm:py-14">
            <div className="mb-6 sm:mb-8">
                <Kicker className="mb-2">Account</Kicker>
                <h1 className="m-0 font-(--font-heading) font-bold text-[26px] text-foreground leading-[1.15] tracking-[-0.02em] sm:text-[36px]">Settings</h1>
                <p className="m-0 mt-1.5 max-w-[60ch] font-sans text-[14px] text-muted-foreground leading-[1.55] sm:text-[15px]">Manage your account, profile visibility, and app appearance. Changes save automatically.</p>
            </div>

            <div className="grid grid-cols-1 items-start gap-5 sm:grid-cols-[220px_1fr] sm:gap-8">
                <SettingsNav nav={nav} active={active} onChange={onChange} />
                <div className="min-w-0">{children}</div>
            </div>
        </main>
    );
}

function SettingsNav({ nav, active, onChange }: { nav: ISettingsNavItem[]; active: SettingsSectionId; onChange: (id: SettingsSectionId) => void }) {
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

    return (
        <nav
            aria-label="Settings sections"
            className={cn("-mx-4 sm:mx-0", "sticky top-14 z-20 border-border border-b bg-background/85 backdrop-blur-md backdrop-saturate-150 supports-[backdrop-filter]:bg-background/65", "sm:relative sm:top-20 sm:z-0 sm:border-0 sm:bg-transparent sm:backdrop-blur-none sm:backdrop-saturate-100")}
        >
            <div ref={scrollerRef} className="flex gap-1 overflow-x-auto px-3 py-2 [scrollbar-width:none] sm:flex-col sm:gap-0.5 sm:overflow-visible sm:p-0 [&::-webkit-scrollbar]:hidden">
                {nav.map(({ id, label, Icon }) => {
                    const isActive = active === id;
                    return (
                        <button
                            key={id}
                            ref={isActive ? activeRef : undefined}
                            type="button"
                            onClick={() => onChange(id)}
                            aria-current={isActive ? "page" : undefined}
                            className={cn(
                                "inline-flex shrink-0 cursor-pointer items-center gap-2 whitespace-nowrap rounded-lg border border-transparent px-3 py-2 text-left font-sans text-[13.5px] outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background",
                                "min-h-9 touch-manipulation",
                                "sm:w-full sm:py-2 sm:text-[14px]",
                                isActive ? "bg-accent font-semibold text-foreground" : "text-muted-foreground hover:bg-accent/60 hover:text-foreground",
                            )}
                        >
                            <Icon className="size-4 shrink-0 opacity-85" />
                            <span className="min-w-0">{label}</span>
                        </button>
                    );
                })}
            </div>
        </nav>
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

export function SectionLabel({ icon, children }: { icon?: ReactNode; children: ReactNode }) {
    return (
        <div className="mb-2.5 flex items-center gap-2 font-sans font-semibold text-[11px] text-muted-foreground uppercase tracking-[0.16em]">
            {icon ? <span className="inline-flex size-3.5 items-center justify-center [&_svg]:size-3.5">{icon}</span> : null}
            {children}
        </div>
    );
}

export function Mono({ children }: { children: ReactNode }) {
    return <code className="font-mono text-[12px] text-foreground/85">{children}</code>;
}
