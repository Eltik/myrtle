import { BoxesIcon, ChevronDownIcon, MapIcon } from "lucide-react";
import type { ReactNode } from "react";
import { Sidebar, SidebarContent, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarMenuSub, SidebarMenuSubButton, SidebarMenuSubItem, SidebarProvider } from "frontend";

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

export const NestedStages = () => (
    <Panel>
        <Head subtitle="Chapter 8 — Roaring Flare" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Stages</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <MapIcon />
                                    <span>Chapter 8</span>
                                    <ChevronDownIcon className="ml-auto" />
                                </SidebarMenuButton>
                                <SidebarMenuSub>
                                    <SidebarMenuSubItem>
                                        <SidebarMenuSubButton>
                                            <span>8-1 Twist of Fate</span>
                                        </SidebarMenuSubButton>
                                    </SidebarMenuSubItem>
                                    <SidebarMenuSubItem>
                                        <SidebarMenuSubButton isActive>
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
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </Panel>
);

export const WithTrailingCounts = () => (
    <Panel>
        <Head subtitle="Depot · 1,284 items" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Materials</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <BoxesIcon />
                                    <span>Tier 4</span>
                                    <ChevronDownIcon className="ml-auto" />
                                </SidebarMenuButton>
                                <SidebarMenuSub>
                                    {[
                                        { count: "12", label: "Chip Catalyst" },
                                        { count: "3", label: "Bipolar Nanoflake" },
                                        { count: "7", label: "D32 Steel" },
                                    ].map((mat) => (
                                        <SidebarMenuSubItem key={mat.label}>
                                            <SidebarMenuSubButton size="sm">
                                                <span>{mat.label}</span>
                                                <span className="ml-auto text-muted-foreground tabular-nums">{mat.count}</span>
                                            </SidebarMenuSubButton>
                                        </SidebarMenuSubItem>
                                    ))}
                                </SidebarMenuSub>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </Panel>
);
