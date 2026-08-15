import { ChevronDownIcon, LayersIcon, MapIcon } from "lucide-react";
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

export const ChapterStages = () => (
    <Panel>
        <Head subtitle="Chapter 8 — Roaring Flare" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Main theme</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton isActive>
                                    <MapIcon />
                                    <span>Chapter 8</span>
                                    <ChevronDownIcon className="ml-auto" />
                                </SidebarMenuButton>
                                <SidebarMenuSub>
                                    {[
                                        { active: true, label: "8-1 Twist of Fate" },
                                        { active: false, label: "8-7 Ashen Snow" },
                                        { active: false, label: "8-16 Roaring Flare" },
                                        { active: false, label: "S8-4 Vestige" },
                                    ].map((stage) => (
                                        <SidebarMenuSubItem key={stage.label}>
                                            <SidebarMenuSubButton isActive={stage.active}>
                                                <span>{stage.label}</span>
                                            </SidebarMenuSubButton>
                                        </SidebarMenuSubItem>
                                    ))}
                                </SidebarMenuSub>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <MapIcon />
                                    <span>Chapter 9</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </Panel>
);

export const PlannerBreakdown = () => (
    <Panel>
        <Head subtitle="Upgrade planner" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Queue</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <LayersIcon />
                                    <span>Mlynar</span>
                                    <ChevronDownIcon className="ml-auto" />
                                </SidebarMenuButton>
                                <SidebarMenuSub>
                                    <SidebarMenuSubItem>
                                        <SidebarMenuSubButton size="sm">
                                            <span>E2 60 → 90</span>
                                        </SidebarMenuSubButton>
                                    </SidebarMenuSubItem>
                                    <SidebarMenuSubItem>
                                        <SidebarMenuSubButton isActive size="sm">
                                            <span>S3 M1 → M3</span>
                                        </SidebarMenuSubButton>
                                    </SidebarMenuSubItem>
                                </SidebarMenuSub>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <LayersIcon />
                                    <span>Muelsyse</span>
                                    <ChevronDownIcon className="ml-auto" />
                                </SidebarMenuButton>
                                <SidebarMenuSub>
                                    <SidebarMenuSubItem>
                                        <SidebarMenuSubButton size="sm">
                                            <span>Module ELM-Y</span>
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
