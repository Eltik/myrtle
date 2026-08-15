import { ChevronsUpDownIcon, MapIcon, SearchIcon, UsersIcon } from "lucide-react";
import type { ReactNode } from "react";
import { Sidebar, SidebarContent, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarInput, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarProvider, SidebarSeparator } from "frontend";

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
                        <SidebarMenuItem>
                            <SidebarMenuButton isActive>
                                <UsersIcon />
                                <span>Operators</span>
                            </SidebarMenuButton>
                        </SidebarMenuItem>
                        <SidebarMenuItem>
                            <SidebarMenuButton>
                                <MapIcon />
                                <span>Stages</span>
                            </SidebarMenuButton>
                        </SidebarMenuItem>
                    </SidebarMenu>
                </SidebarGroupContent>
            </SidebarGroup>
        </SidebarContent>
    </div>
);

export const BrandBlock = () => (
    <Panel>
        <SidebarHeader>
            <div className="flex items-center gap-2 px-1 py-1.5">
                <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary font-semibold text-primary-foreground text-sm">M</div>
                <div className="grid flex-1 leading-tight">
                    <span className="truncate font-semibold text-sm">Myrtle</span>
                    <span className="truncate text-muted-foreground text-xs">Doctor Kal'tsit · EN</span>
                </div>
            </div>
        </SidebarHeader>
        <Nav />
    </Panel>
);

export const WithServerSwitcher = () => (
    <Panel>
        <SidebarHeader>
            <SidebarMenu>
                <SidebarMenuItem>
                    <SidebarMenuButton size="lg" variant="outline">
                        <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary font-semibold text-primary-foreground text-sm">EN</div>
                        <div className="grid flex-1 leading-tight">
                            <span className="truncate font-medium text-sm">Global server</span>
                            <span className="truncate text-muted-foreground text-xs">Switch to CN · JP</span>
                        </div>
                        <ChevronsUpDownIcon className="ml-auto" />
                    </SidebarMenuButton>
                </SidebarMenuItem>
            </SidebarMenu>
        </SidebarHeader>
        <Nav />
    </Panel>
);

export const WithSearch = () => (
    <Panel>
        <SidebarHeader>
            <div className="flex items-center gap-2 px-1 py-1">
                <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary font-semibold text-primary-foreground text-sm">M</div>
                <span className="truncate font-semibold text-sm">Myrtle</span>
            </div>
            <div className="relative">
                <SearchIcon className="pointer-events-none absolute top-2 left-2 z-10 size-4 text-muted-foreground" />
                <SidebarInput className="pl-8" placeholder="Search operators…" />
            </div>
            <SidebarSeparator className="mx-0" />
        </SidebarHeader>
        <Nav />
    </Panel>
);
