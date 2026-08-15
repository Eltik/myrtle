import { BoxesIcon, ChevronsUpDownIcon, FilterIcon, LayersIcon, MapIcon, SparklesIcon, UsersIcon } from "lucide-react";
import type { ReactNode } from "react";
import { Sidebar, SidebarContent, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarProvider } from "frontend";

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

export const ActiveAndDisabled = () => (
    <Panel>
        <Head subtitle="Doctor Kal'tsit · EN" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Navigation</SidebarGroupLabel>
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
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <SparklesIcon />
                                    <span>Recruitment</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton disabled>
                                    <BoxesIcon />
                                    <span>Depot (syncing…)</span>
                                </SidebarMenuButton>
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
        <Head subtitle="Upgrade planner" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Large — account row</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton size="lg">
                                    <img alt="Mlynar" className="size-8 shrink-0 rounded-lg object-cover" src="https://api.myrtle.moe/api/avatar/char_4064_mlynar" />
                                    <div className="grid flex-1 leading-tight">
                                        <span className="truncate font-medium text-sm">Dr. Kal'tsit</span>
                                        <span className="truncate text-muted-foreground text-xs">231 operators</span>
                                    </div>
                                    <ChevronsUpDownIcon className="ml-auto" />
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
                <SidebarGroup>
                    <SidebarGroupLabel>Default</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton isActive>
                                    <LayersIcon />
                                    <span>Upgrade planner</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
                <SidebarGroup>
                    <SidebarGroupLabel>Small — saved filters</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton isActive size="sm">
                                    <FilterIcon />
                                    <span>6★ · not E2</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton size="sm">
                                    <FilterIcon />
                                    <span>Modules pending</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </Panel>
);

export const OutlineVariant = () => (
    <Panel>
        <Head subtitle="Global server" />
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Account</SidebarGroupLabel>
                    <SidebarGroupContent>
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
                    </SidebarGroupContent>
                </SidebarGroup>
                <SidebarGroup>
                    <SidebarGroupLabel>Links</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton render={<a href="/operators" />}>
                                    <UsersIcon />
                                    <span>Operator index</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                            <SidebarMenuItem>
                                <SidebarMenuButton variant="outline">
                                    <MapIcon />
                                    <span>Stage lookup</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </Panel>
);
