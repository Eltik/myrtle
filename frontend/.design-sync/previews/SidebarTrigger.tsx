import { BoxesIcon, MapIcon, SparklesIcon, UsersIcon } from "lucide-react";
import type { ReactNode } from "react";
import { Sidebar, SidebarContent, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarInset, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarProvider, SidebarTrigger } from "frontend";

const ROSTER = [
    { icon: UsersIcon, label: "Operators" },
    { icon: MapIcon, label: "Stages" },
    { icon: SparklesIcon, label: "Recruitment" },
    { icon: BoxesIcon, label: "Depot" },
];

const Nav = () => (
    <>
        <SidebarHeader>
            <div className="flex items-center gap-2 px-1 py-1.5">
                <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary font-semibold text-primary-foreground text-sm">M</div>
                <div className="grid flex-1 leading-tight">
                    <span className="truncate font-semibold text-sm">Myrtle</span>
                    <span className="truncate text-muted-foreground text-xs">Doctor Kal'tsit · EN</span>
                </div>
            </div>
        </SidebarHeader>
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Roster</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            {ROSTER.map((item, i) => (
                                <SidebarMenuItem key={item.label}>
                                    <SidebarMenuButton isActive={i === 0} tooltip={item.label}>
                                        <item.icon />
                                        <span>{item.label}</span>
                                    </SidebarMenuButton>
                                </SidebarMenuItem>
                            ))}
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </>
);

const Shell = ({ children, open }: { children: ReactNode; open?: boolean }) => (
    <SidebarProvider defaultOpen={open ?? true}>
        <Sidebar collapsible="icon">
            <Nav />
        </Sidebar>
        {children}
    </SidebarProvider>
);

export const InPageHeader = () => (
    <Shell>
        <SidebarInset>
            <header className="flex h-12 items-center gap-2 border-b px-4">
                <SidebarTrigger />
                <span className="font-medium text-sm">Operators</span>
                <span className="ml-auto text-muted-foreground text-xs tabular-nums">231 owned</span>
            </header>
            <div className="grid gap-2 p-4 sm:grid-cols-3">
                {[
                    { detail: "E2 60 · Guard", name: "Mlynar" },
                    { detail: "E2 42 · Guard", name: "Skadi the Corrupting Heart" },
                    { detail: "E2 55 · Vanguard", name: "Texas" },
                ].map((op) => (
                    <div className="rounded-lg border p-3" key={op.name}>
                        <div className="font-medium text-sm">{op.name}</div>
                        <div className="text-muted-foreground text-xs">{op.detail}</div>
                    </div>
                ))}
            </div>
        </SidebarInset>
    </Shell>
);

export const WithCollapsedSidebar = () => (
    <Shell open={false}>
        <SidebarInset>
            <header className="flex h-12 items-center gap-2 border-b px-4">
                <SidebarTrigger />
                <span className="font-medium text-sm">Chapter 8 — Roaring Flare</span>
                <span className="ml-auto text-muted-foreground text-xs tabular-nums">14 stages · 3 CM cleared</span>
            </header>
            <div className="grid gap-2 p-4 sm:grid-cols-3">
                {[
                    { detail: "18 sanity · 3★ cleared", name: "8-1 Twist of Fate" },
                    { detail: "21 sanity · not cleared", name: "8-7 Ashen Snow" },
                    { detail: "24 sanity · 3★ cleared", name: "S8-4 Vestige" },
                ].map((stage) => (
                    <div className="rounded-lg border p-3" key={stage.name}>
                        <div className="font-medium text-sm">{stage.name}</div>
                        <div className="text-muted-foreground text-xs tabular-nums">{stage.detail}</div>
                    </div>
                ))}
            </div>
        </SidebarInset>
    </Shell>
);

export const WithLabel = () => (
    <Shell>
        <SidebarInset>
            <header className="flex h-12 items-center gap-2 border-b px-4">
                <SidebarTrigger className="size-8" />
                <span className="font-medium text-sm">Upgrade planner</span>
                <span className="ml-auto text-muted-foreground text-xs">Toggle with ⌘ \</span>
            </header>
            <div className="grid gap-2 p-4 sm:grid-cols-2">
                {[
                    { detail: "12 Chip Catalysts short", name: "Mlynar — S3 M3" },
                    { detail: "Ready to promote", name: "Muelsyse — E2 60" },
                ].map((row) => (
                    <div className="rounded-lg border p-3" key={row.name}>
                        <div className="font-medium text-sm">{row.name}</div>
                        <div className="text-muted-foreground text-xs">{row.detail}</div>
                    </div>
                ))}
            </div>
        </SidebarInset>
    </Shell>
);
