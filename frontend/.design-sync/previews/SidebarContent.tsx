import { BoxesIcon, HammerIcon, LayersIcon, MapIcon, SparklesIcon, TrophyIcon, UsersIcon } from "lucide-react";
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

export const ScrollingSections = () => (
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
                                    <BoxesIcon />
                                    <span>Depot</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
                <SidebarGroup>
                    <SidebarGroupLabel>Progression</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <MapIcon />
                                    <span>Stages</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <LayersIcon />
                                    <span>Upgrade planner</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <HammerIcon />
                                    <span>Base layout</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <TrophyIcon />
                                    <span>Tier lists</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </Panel>
);

export const SingleSection = () => (
    <Panel>
        <Head subtitle="Depot · 1,284 items" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Materials</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            {[
                                { count: "12", label: "Chip Catalyst" },
                                { count: "348", label: "Orirock Cube" },
                                { count: "3", label: "Bipolar Nanoflake" },
                                { count: "7", label: "D32 Steel" },
                            ].map((item, i) => (
                                <SidebarMenuItem key={item.label}>
                                    <SidebarMenuButton isActive={i === 0}>
                                        <BoxesIcon />
                                        <span>{item.label}</span>
                                    </SidebarMenuButton>
                                    <SidebarMenuBadge>{item.count}</SidebarMenuBadge>
                                </SidebarMenuItem>
                            ))}
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </Panel>
);
