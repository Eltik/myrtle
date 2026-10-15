import { Link } from "@tanstack/react-router";
import { useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Sheet, SheetDescription, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { useAuth } from "#/hooks/use-auth";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./AdminBar.messages";
import { type IAdminNav, type IAdminNavSection, useRoleLabel } from "./nav";

function CountBadge({ section, variant }: { section: IAdminNavSection; variant?: IAdminNavSection["countVariant"] }): React.ReactElement | null {
    const f = useFormatters();
    if (!section.count) return null;
    return (
        <Badge size="sm" variant={variant ?? section.countVariant} className="tabular-nums">
            {f.number(section.count)}
        </Badge>
    );
}

/**
 * The admin section bar under the site header. Both layouts are rendered and
 * swapped by the `lg` CSS breakpoint (no JS viewport state, so the server
 * render matches): the full underline tab bar on wide screens, a single
 * section button that opens a left sheet below that.
 */
export function AdminBar({ nav }: { nav: IAdminNav }): React.ReactElement {
    return (
        <>
            <FullBar nav={nav} />
            <CompactBar nav={nav} />
        </>
    );
}

function FullBar({ nav }: { nav: IAdminNav }): React.ReactElement {
    const t: TypedT<typeof messages> = useT("admin");
    return (
        <div className="hidden border-border border-b bg-card lg:block">
            <div className="mx-auto flex min-h-[52px] max-w-[1320px] flex-nowrap items-center gap-x-5 px-8 xl:px-10">
                <span className="shrink-0 font-bold text-[11px] text-primary uppercase tracking-[0.22em]">{t("shell.kicker")}</span>
                <div className="min-w-0 max-w-full overflow-x-auto overflow-y-hidden">
                    <Tabs value={nav.current}>
                        <TabsList variant="underline" aria-label={t("shell.nav.label")}>
                            {nav.sections.map((s) => (
                                <TabsTab key={s.id} value={s.id} nativeButton={false} render={<Link to={s.to} />}>
                                    {s.label}
                                    <CountBadge section={s} />
                                </TabsTab>
                            ))}
                        </TabsList>
                    </Tabs>
                </div>
            </div>
        </div>
    );
}

function CompactBar({ nav }: { nav: IAdminNav }): React.ReactElement {
    const t: TypedT<typeof messages> = useT("admin");
    const roleLabel = useRoleLabel();
    const { user } = useAuth();
    const [open, setOpen] = useState(false);
    const current = nav.sections.find((s) => s.id === nav.current);

    return (
        <div className="flex items-center gap-2.5 border-border border-b bg-card px-4 py-2 md:px-8 lg:hidden">
            <button type="button" aria-haspopup="dialog" aria-expanded={open} onClick={() => setOpen(true)} className="flex h-10 min-w-0 flex-1 cursor-pointer items-center gap-2.5 rounded-[10px] border border-input bg-background pr-3 pl-3.5 text-foreground transition-colors hover:bg-accent">
                <span className="font-bold text-[11px] text-primary uppercase tracking-[0.18em]">{t("shell.kicker")}</span>
                <span className="h-4 w-px bg-border" />
                <span className="truncate font-medium text-[14px]">{nav.currentLabel}</span>
                {current ? <CountBadge section={current} variant="default" /> : null}
                <span className="flex-1" />
                <span className="shrink-0 text-[12.5px] text-muted-foreground">{t("shell.compact.sections", { count: nav.sections.length })}</span>
                <span aria-hidden className="text-[12px] text-muted-foreground">
                    ▾
                </span>
            </button>
            <Sheet open={open} onOpenChange={setOpen}>
                <SheetPopup side="left">
                    <SheetHeader>
                        <SheetTitle>{t("shell.sheet.title")}</SheetTitle>
                        <SheetDescription>{t("shell.sheet.description", { nickname: user?.nickname ?? user?.uid ?? "", role: roleLabel(user?.role) })}</SheetDescription>
                    </SheetHeader>
                    <SheetPanel>
                        <nav aria-label={t("shell.nav.label")} className="flex flex-col gap-0.5">
                            {nav.sections.map((s) => {
                                const active = s.id === nav.current;
                                return (
                                    <Link
                                        key={s.id}
                                        to={s.to}
                                        onClick={() => setOpen(false)}
                                        aria-current={active ? "page" : undefined}
                                        className={cn("flex min-h-11 items-center gap-2.5 rounded-lg px-3 text-left text-[15px] text-foreground transition-colors hover:bg-accent", active ? "bg-accent font-semibold" : "bg-transparent font-normal")}
                                    >
                                        <span className="flex-1">{s.label}</span>
                                        <CountBadge section={s} />
                                    </Link>
                                );
                            })}
                        </nav>
                    </SheetPanel>
                </SheetPopup>
            </Sheet>
        </div>
    );
}
