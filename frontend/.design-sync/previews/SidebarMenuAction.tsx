import { BoxesIcon, ChevronsUpDownIcon, LayersIcon, MapIcon, MoreHorizontalIcon, PinIcon, PlusIcon, UsersIcon } from "lucide-react";
import type { ReactNode } from "react";
import { Sidebar, SidebarContent, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuAction, SidebarMenuButton, SidebarMenuItem, SidebarProvider } from "frontend";

const Panel = ({ children }: { children: ReactNode }) => (
    <SidebarProvider className="h-96 min-h-0 w-64 overflow-hidden rounded-xl border">
        <Sidebar collapsible="none">{children}</Sidebar>
    </SidebarProvider>
);

const Head = ({ subtitle }: { subtitle: string }) => (
    <SidebarHeader>
        <div className="flex items-center gap-2 px-1 py-1.5">
            <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary font-semibold text-primary-foreground text-sm">M</div>
            <div className="grid flex-1 leading-tight">
                <span className="truncate font-semibold text-sm">Myrtle</span>
                <span className="truncate text-muted-foreground text-xs">{subtitle}</span>
            </div>
        </div>
    </SidebarHeader>
);

export const RowActions = () => (
    <Panel>
        <Head subtitle="Doctor Kal'tsit · EN" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Pinned</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton isActive>
                                    <UsersIcon />
                                    <span>Operators</span>
                                </SidebarMenuButton>
                                <SidebarMenuAction title="More options">
                                    <MoreHorizontalIcon />
                                    <span className="sr-only">More options</span>
                                </SidebarMenuAction>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <MapIcon />
                                    <span>Chapter 8</span>
                                </SidebarMenuButton>
                                <SidebarMenuAction title="Unpin chapter">
                                    <PinIcon />
                                    <span className="sr-only">Unpin chapter</span>
                                </SidebarMenuAction>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <BoxesIcon />
                                    <span>Depot</span>
                                </SidebarMenuButton>
                                <SidebarMenuAction render={<a href="/depot/new" />} title="Add material">
                                    <PlusIcon />
                                    <span className="sr-only">Add material</span>
                                </SidebarMenuAction>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </Panel>
);

export const OnLargeRow = () => (
    <Panel>
        <Head subtitle="Upgrade planner" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Queue</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton size="lg">
                                    <img alt="Mlynar" className="size-8 shrink-0 rounded-lg object-cover" src="https://api.myrtle.moe/api/avatar/char_4064_mlynar" />
                                    <div className="grid flex-1 leading-tight">
                                        <span className="truncate font-medium text-sm">Mlynar</span>
                                        <span className="truncate text-muted-foreground text-xs">S3 M3 · 12 mats short</span>
                                    </div>
                                </SidebarMenuButton>
                                <SidebarMenuAction title="Queue options">
                                    <MoreHorizontalIcon />
                                    <span className="sr-only">Queue options</span>
                                </SidebarMenuAction>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton size="sm">
                                    <LayersIcon />
                                    <span>Texas — Module X</span>
                                </SidebarMenuButton>
                                <SidebarMenuAction title="Reorder">
                                    <ChevronsUpDownIcon />
                                    <span className="sr-only">Reorder</span>
                                </SidebarMenuAction>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </Panel>
);
