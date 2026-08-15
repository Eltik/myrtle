import { ChevronDownIcon, MapIcon, StarIcon } from "lucide-react";
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

export const ActiveStage = () => (
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
                                        <SidebarMenuSubButton href="/stages/8-1">
                                            <span>8-1 Twist of Fate</span>
                                        </SidebarMenuSubButton>
                                    </SidebarMenuSubItem>
                                    <SidebarMenuSubItem>
                                        <SidebarMenuSubButton href="/stages/8-7" isActive>
                                            <StarIcon />
                                            <span>8-7 Ashen Snow</span>
                                        </SidebarMenuSubButton>
                                    </SidebarMenuSubItem>
                                    <SidebarMenuSubItem>
                                        <SidebarMenuSubButton aria-disabled href="/stages/8-16">
                                            <span>8-16 Roaring Flare</span>
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

export const Sizes = () => (
    <Panel>
        <Head subtitle="Farming routes" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Medium rows</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <MapIcon />
                                    <span>Annihilation</span>
                                </SidebarMenuButton>
                                <SidebarMenuSub>
                                    <SidebarMenuSubItem>
                                        <SidebarMenuSubButton isActive>
                                            <span>Chernobog · 400 kills</span>
                                        </SidebarMenuSubButton>
                                    </SidebarMenuSubItem>
                                    <SidebarMenuSubItem>
                                        <SidebarMenuSubButton>
                                            <span>Lungmen Outskirts</span>
                                        </SidebarMenuSubButton>
                                    </SidebarMenuSubItem>
                                </SidebarMenuSub>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
                <SidebarGroup>
                    <SidebarGroupLabel>Small rows</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton size="sm">
                                    <MapIcon />
                                    <span>Supply</span>
                                </SidebarMenuButton>
                                <SidebarMenuSub>
                                    <SidebarMenuSubItem>
                                        <SidebarMenuSubButton size="sm">
                                            <span>CE-6 · 30 sanity</span>
                                        </SidebarMenuSubButton>
                                    </SidebarMenuSubItem>
                                    <SidebarMenuSubItem>
                                        <SidebarMenuSubButton size="sm">
                                            <span>LS-6 · 30 sanity</span>
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
