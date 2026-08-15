import { BoxesIcon, LayersIcon, MapIcon, UsersIcon } from "lucide-react";
import { Sidebar, SidebarContent, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarInset, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarProvider, SidebarRail, SidebarTrigger } from "frontend";

const ROSTER = [
    { icon: UsersIcon, label: "Operators" },
    { icon: MapIcon, label: "Stages" },
    { icon: BoxesIcon, label: "Depot" },
    { icon: LayersIcon, label: "Upgrade planner" },
];

const Nav = ({ title }: { title: string }) => (
    <>
        <SidebarHeader>
            <div className="flex items-center gap-2 px-1 py-1.5">
                <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary font-semibold text-primary-foreground text-sm">M</div>
                <div className="grid flex-1 leading-tight">
                    <span className="truncate font-semibold text-sm">Myrtle</span>
                    <span className="truncate text-muted-foreground text-xs">{title}</span>
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

const Workspace = ({ heading }: { heading: string }) => (
    <SidebarInset>
        <header className="flex h-12 items-center gap-2 border-b px-4">
            <SidebarTrigger />
            <span className="font-medium text-sm">{heading}</span>
        </header>
        <div className="p-4">
            <p className="text-muted-foreground text-sm">Drag the rail on the sidebar edge to collapse it, or press ⌘ \.</p>
            <div className="mt-3 grid gap-2 sm:grid-cols-3">
                {[
                    { detail: "18 sanity · 3★ cleared", name: "8-1 Twist of Fate" },
                    { detail: "21 sanity · not cleared", name: "8-7 Ashen Snow" },
                    { detail: "30 sanity · 3★ cleared", name: "CE-6 Cargo Escort" },
                ].map((stage) => (
                    <div className="rounded-lg border p-3" key={stage.name}>
                        <div className="font-medium text-sm">{stage.name}</div>
                        <div className="text-muted-foreground text-xs tabular-nums">{stage.detail}</div>
                    </div>
                ))}
            </div>
        </div>
    </SidebarInset>
);

export const EdgeHandle = () => (
    <SidebarProvider>
        <Sidebar collapsible="icon">
            <Nav title="Doctor Kal'tsit · EN" />
            <SidebarRail />
        </Sidebar>
        <Workspace heading="Operators" />
    </SidebarProvider>
);

export const RightSide = () => (
    <SidebarProvider>
        <Workspace heading="Depot" />
        <Sidebar collapsible="icon" side="right">
            <Nav title="Depot · 1,284 items" />
            <SidebarRail />
        </Sidebar>
    </SidebarProvider>
);
