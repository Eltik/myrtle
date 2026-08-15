import { BoxesIcon, MapIcon, UsersIcon } from "lucide-react";
import type { ReactNode } from "react";
import { Badge, Sidebar, SidebarContent, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarInset, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarProvider, SidebarTrigger } from "frontend";

const ROSTER = [
    { icon: UsersIcon, label: "Operators" },
    { icon: MapIcon, label: "Stages" },
    { icon: BoxesIcon, label: "Depot" },
];

const OPERATORS = [
    { detail: "E2 60 · Guard", id: "char_4064_mlynar", name: "Mlynar", trust: "200%" },
    { detail: "E2 42 · Guard", id: "char_263_skadi", name: "Skadi the Corrupting Heart", trust: "164%" },
    { detail: "E2 55 · Vanguard", id: "char_102_texas", name: "Texas", trust: "189%" },
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
                                    <SidebarMenuButton isActive={i === 0}>
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

const Shell = ({ children, variant }: { children: ReactNode; variant?: "sidebar" | "inset" }) => (
    <SidebarProvider>
        <Sidebar collapsible="icon" variant={variant ?? "sidebar"}>
            <Nav />
        </Sidebar>
        {children}
    </SidebarProvider>
);

export const WorkspaceHeader = () => (
    <Shell>
        <SidebarInset>
            <header className="flex h-12 items-center gap-2 border-b px-4">
                <SidebarTrigger />
                <span className="font-medium text-sm">Operators</span>
                <Badge className="ml-2" variant="secondary">
                    42 at E2
                </Badge>
                <span className="ml-auto text-muted-foreground text-xs tabular-nums">Updated 4 minutes ago</span>
            </header>
            <div className="grid gap-2 p-4">
                {OPERATORS.map((op) => (
                    <div className="flex items-center gap-3 rounded-lg border p-2" key={op.id}>
                        <img alt={op.name} className="size-10 shrink-0 rounded-lg object-cover" src={`https://api.myrtle.moe/api/avatar/${op.id}`} />
                        <div className="grid leading-tight">
                            <span className="font-medium text-sm">{op.name}</span>
                            <span className="text-muted-foreground text-xs">{op.detail}</span>
                        </div>
                        <span className="ml-auto text-muted-foreground text-xs tabular-nums">Trust {op.trust}</span>
                    </div>
                ))}
            </div>
        </SidebarInset>
    </Shell>
);

export const InsetVariant = () => (
    <Shell variant="inset">
        <SidebarInset>
            <header className="flex h-12 items-center gap-2 border-b px-4">
                <SidebarTrigger />
                <span className="font-medium text-sm">Chapter 8 — Roaring Flare</span>
                <span className="ml-auto text-muted-foreground text-xs tabular-nums">14 stages</span>
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
