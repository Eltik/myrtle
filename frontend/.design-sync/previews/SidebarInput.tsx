import { MapIcon, SearchIcon, UsersIcon } from "lucide-react";
import type { ReactNode } from "react";
import { Sidebar, SidebarContent, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarInput, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarProvider } from "frontend";

const Panel = ({ children }: { children: ReactNode }) => (
    <SidebarProvider className="h-96 min-h-0 w-64 overflow-hidden rounded-xl border">
        <Sidebar collapsible="none">{children}</Sidebar>
    </SidebarProvider>
);

const Results = ({ items, label }: { items: string[]; label: string }) => (
    <div className="flex min-h-0 flex-1 flex-col">
        <SidebarContent>
            <SidebarGroup>
                <SidebarGroupLabel>{label}</SidebarGroupLabel>
                <SidebarGroupContent>
                    <SidebarMenu>
                        {items.map((item, i) => (
                            <SidebarMenuItem key={item}>
                                <SidebarMenuButton isActive={i === 0}>
                                    <UsersIcon />
                                    <span>{item}</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                        ))}
                    </SidebarMenu>
                </SidebarGroupContent>
            </SidebarGroup>
        </SidebarContent>
    </div>
);

export const FilterRoster = () => (
    <Panel>
        <SidebarHeader>
            <div className="relative">
                <SearchIcon className="pointer-events-none absolute top-2 left-2 z-10 size-4 text-muted-foreground" />
                <SidebarInput className="pl-8" placeholder="Search operators…" />
            </div>
        </SidebarHeader>
        <Results items={["Mlynar", "Skadi the Corrupting Heart", "Texas", "Muelsyse"]} label="Roster" />
    </Panel>
);

export const WithQuery = () => (
    <Panel>
        <SidebarHeader>
            <div className="relative">
                <SearchIcon className="pointer-events-none absolute top-2 left-2 z-10 size-4 text-muted-foreground" />
                <SidebarInput className="pl-8" defaultValue="skadi" placeholder="Search operators…" />
            </div>
            <span className="px-2 text-muted-foreground text-xs tabular-nums">2 of 231 operators</span>
        </SidebarHeader>
        <Results items={["Skadi", "Skadi the Corrupting Heart"]} label="Matches" />
    </Panel>
);

export const Disabled = () => (
    <Panel>
        <SidebarHeader>
            <div className="relative">
                <SearchIcon className="pointer-events-none absolute top-2 left-2 z-10 size-4 text-muted-foreground" />
                <SidebarInput className="pl-8" disabled placeholder="Roster syncing…" />
            </div>
            <span className="px-2 text-muted-foreground text-xs">Search is unavailable while the depot syncs.</span>
        </SidebarHeader>
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Browse</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton>
                                    <MapIcon />
                                    <span>Stages</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </div>
    </Panel>
);
