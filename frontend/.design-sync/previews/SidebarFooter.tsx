import { BoxesIcon, ChevronsUpDownIcon, MapIcon, RefreshCwIcon, UsersIcon } from "lucide-react";
import type { ReactNode } from "react";
import { Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarProvider, SidebarSeparator } from "frontend";

const NAV = [
    { icon: UsersIcon, label: "Operators" },
    { icon: MapIcon, label: "Stages" },
    { icon: BoxesIcon, label: "Depot" },
];

const Panel = ({ children }: { children: ReactNode }) => (
    <SidebarProvider className="h-96 min-h-0 w-64 overflow-hidden rounded-xl border">
        <Sidebar collapsible="none">{children}</Sidebar>
    </SidebarProvider>
);

const Nav = () => (
    <div className="flex min-h-0 flex-1 flex-col">
        <SidebarContent>
            <SidebarGroup>
                <SidebarGroupLabel>Roster</SidebarGroupLabel>
                <SidebarGroupContent>
                    <SidebarMenu>
                        {NAV.map((item, i) => (
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
);

const Head = () => (
    <SidebarHeader>
        <div className="flex items-center gap-2 px-1 py-1.5">
            <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary font-semibold text-primary-foreground text-sm">M</div>
            <div className="grid flex-1 leading-tight">
                <span className="truncate font-semibold text-sm">Myrtle</span>
                <span className="truncate text-muted-foreground text-xs">EN server</span>
            </div>
        </div>
    </SidebarHeader>
);

export const UserCard = () => (
    <Panel>
        <Head />
        <Nav />
        <SidebarFooter>
            <SidebarMenu>
                <SidebarMenuItem>
                    <SidebarMenuButton size="lg">
                        <img alt="Mlynar" className="size-8 shrink-0 rounded-lg object-cover" src="https://api.myrtle.moe/api/avatar/char_4064_mlynar" />
                        <div className="grid flex-1 leading-tight">
                            <span className="truncate font-medium text-sm">Dr. Kal'tsit</span>
                            <span className="truncate text-muted-foreground text-xs">231 operators</span>
                        </div>
                        <ChevronsUpDownIcon className="ml-auto" />
                    </SidebarMenuButton>
                </SidebarMenuItem>
            </SidebarMenu>
        </SidebarFooter>
    </Panel>
);

export const SyncStatus = () => (
    <Panel>
        <Head />
        <Nav />
        <SidebarFooter>
            <SidebarSeparator className="mx-0" />
            <div className="flex items-center gap-2 px-2 py-1 text-muted-foreground text-xs">
                <RefreshCwIcon className="size-3.5 shrink-0" />
                <span className="truncate">Synced 4 minutes ago</span>
            </div>
            <div className="flex items-center justify-between px-2 pb-1 text-xs">
                <span className="text-muted-foreground">Sanity</span>
                <span className="font-medium tabular-nums">128 / 135</span>
            </div>
        </SidebarFooter>
    </Panel>
);
