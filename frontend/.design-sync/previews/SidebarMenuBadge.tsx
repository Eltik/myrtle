import { BoxesIcon, MapIcon, SparklesIcon, UsersIcon } from "lucide-react";
import type { ReactNode } from "react";
import { Sidebar, SidebarContent, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuBadge, SidebarMenuButton, SidebarMenuItem, SidebarProvider } from "frontend";

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

export const Counts = () => (
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
                                    <SparklesIcon />
                                    <span>Recruitment</span>
                                </SidebarMenuButton>
                                <SidebarMenuBadge>6</SidebarMenuBadge>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <MapIcon />
                                    <span>Stages</span>
                                </SidebarMenuButton>
                                <SidebarMenuBadge>New</SidebarMenuBadge>
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

export const AcrossRowSizes = () => (
    <Panel>
        <Head subtitle="Depot · 1,284 items" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Materials</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton size="lg">
                                    <img alt="Mlynar" className="size-8 shrink-0 rounded-lg object-cover" src="https://api.myrtle.moe/api/avatar/char_4064_mlynar" />
                                    <div className="grid flex-1 leading-tight">
                                        <span className="truncate font-medium text-sm">Mlynar</span>
                                        <span className="truncate text-muted-foreground text-xs">Materials missing</span>
                                    </div>
                                </SidebarMenuButton>
                                <SidebarMenuBadge>12</SidebarMenuBadge>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <BoxesIcon />
                                    <span>Orirock Cube</span>
                                </SidebarMenuButton>
                                <SidebarMenuBadge>348</SidebarMenuBadge>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton size="sm">
                                    <BoxesIcon />
                                    <span>Bipolar Nanoflake</span>
                                </SidebarMenuButton>
                                <SidebarMenuBadge>3</SidebarMenuBadge>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton size="sm">
                                    <BoxesIcon />
                                    <span>D32 Steel</span>
                                </SidebarMenuButton>
                                <SidebarMenuBadge>7</SidebarMenuBadge>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </Panel>
);
