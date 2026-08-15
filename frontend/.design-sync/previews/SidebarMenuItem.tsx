import { BoxesIcon, ChevronRightIcon, MapIcon, MoreHorizontalIcon, UsersIcon } from "lucide-react";
import type { ReactNode } from "react";
import { Sidebar, SidebarContent, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuAction, SidebarMenuBadge, SidebarMenuButton, SidebarMenuItem, SidebarMenuSub, SidebarMenuSubButton, SidebarMenuSubItem, SidebarProvider } from "frontend";

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

export const NavRows = () => (
    <Panel>
        <Head subtitle="Doctor Kal'tsit · EN" />
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
                                <SidebarMenuBadge>231</SidebarMenuBadge>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <MapIcon />
                                    <span>Stages</span>
                                </SidebarMenuButton>
                                <SidebarMenuAction title="Pin chapter">
                                    <MoreHorizontalIcon />
                                    <span className="sr-only">Pin chapter</span>
                                </SidebarMenuAction>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <BoxesIcon />
                                    <span>Depot</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </Panel>
);

export const ItemWithNestedStages = () => (
    <Panel>
        <Head subtitle="Chapter 8 — Roaring Flare" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Progression</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton isActive>
                                    <MapIcon />
                                    <span>Chapter 8</span>
                                    <ChevronRightIcon className="ml-auto rotate-90" />
                                </SidebarMenuButton>
                                <SidebarMenuSub>
                                    <SidebarMenuSubItem>
                                        <SidebarMenuSubButton isActive>
                                            <span>8-1 Twist of Fate</span>
                                        </SidebarMenuSubButton>
                                    </SidebarMenuSubItem>
                                    <SidebarMenuSubItem>
                                        <SidebarMenuSubButton>
                                            <span>8-7 Ashen Snow</span>
                                        </SidebarMenuSubButton>
                                    </SidebarMenuSubItem>
                                    <SidebarMenuSubItem>
                                        <SidebarMenuSubButton>
                                            <span>S8-4 Vestige</span>
                                        </SidebarMenuSubButton>
                                    </SidebarMenuSubItem>
                                </SidebarMenuSub>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <MapIcon />
                                    <span>Chapter 9</span>
                                </SidebarMenuButton>
                                <SidebarMenuBadge>New</SidebarMenuBadge>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </Panel>
);
