import { BoxesIcon, LayersIcon, PlusIcon, RefreshCwIcon, UsersIcon } from "lucide-react";
import type { ReactNode } from "react";
import { Sidebar, SidebarContent, SidebarGroup, SidebarGroupAction, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuBadge, SidebarMenuButton, SidebarMenuItem, SidebarProvider } from "frontend";

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

export const AddToQueue = () => (
    <Panel>
        <Head subtitle="Upgrade planner" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Upgrade queue</SidebarGroupLabel>
                    <SidebarGroupAction title="Add operator">
                        <PlusIcon />
                        <span className="sr-only">Add operator</span>
                    </SidebarGroupAction>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            {["Mlynar — S3M3", "Muelsyse — E2 60", "Texas — Module X"].map((label) => (
                                <SidebarMenuItem key={label}>
                                    <SidebarMenuButton>
                                        <LayersIcon />
                                        <span>{label}</span>
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

export const RefreshDepot = () => (
    <Panel>
        <Head subtitle="Depot · synced 4 min ago" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Materials</SidebarGroupLabel>
                    <SidebarGroupAction title="Re-scan depot">
                        <RefreshCwIcon />
                        <span className="sr-only">Re-scan depot</span>
                    </SidebarGroupAction>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            {[
                                { count: "12", label: "Chip Catalyst" },
                                { count: "348", label: "Orirock Cube" },
                                { count: "3", label: "Bipolar Nanoflake" },
                            ].map((item) => (
                                <SidebarMenuItem key={item.label}>
                                    <SidebarMenuButton>
                                        <BoxesIcon />
                                        <span>{item.label}</span>
                                    </SidebarMenuButton>
                                    <SidebarMenuBadge>{item.count}</SidebarMenuBadge>
                                </SidebarMenuItem>
                            ))}
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
                <SidebarGroup>
                    <SidebarGroupLabel>Roster</SidebarGroupLabel>
                    <SidebarGroupAction render={<a href="/operators" />} title="Browse all operators">
                        <UsersIcon />
                        <span className="sr-only">Browse all operators</span>
                    </SidebarGroupAction>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <UsersIcon />
                                    <span>Operators</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </Panel>
);
