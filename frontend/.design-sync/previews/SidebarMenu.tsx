import { BoxesIcon, LayersIcon, MapIcon, MoreHorizontalIcon, SparklesIcon, TrophyIcon, UsersIcon } from "lucide-react";
import type { ReactNode } from "react";
import { Sidebar, SidebarContent, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuAction, SidebarMenuBadge, SidebarMenuButton, SidebarMenuItem, SidebarProvider } from "frontend";

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

export const PrimaryNav = () => (
    <Panel>
        <Head subtitle="Doctor Kal'tsit · EN" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Navigation</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            {[
                                { icon: UsersIcon, label: "Operators" },
                                { icon: MapIcon, label: "Stages" },
                                { icon: SparklesIcon, label: "Recruitment" },
                                { icon: BoxesIcon, label: "Depot" },
                                { icon: LayersIcon, label: "Upgrade planner" },
                                { icon: TrophyIcon, label: "Tier lists" },
                            ].map((item, i) => (
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
    </Panel>
);

export const WithBadgesAndActions = () => (
    <Panel>
        <Head subtitle="Depot · 1,284 items" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Watchlist</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton isActive>
                                    <BoxesIcon />
                                    <span>Chip Catalyst</span>
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
                                <SidebarMenuButton>
                                    <BoxesIcon />
                                    <span>D32 Steel</span>
                                </SidebarMenuButton>
                                <SidebarMenuAction title="More">
                                    <MoreHorizontalIcon />
                                    <span className="sr-only">More</span>
                                </SidebarMenuAction>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </Panel>
);
