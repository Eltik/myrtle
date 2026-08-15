import { BoxesIcon, HammerIcon, LayersIcon, MapIcon, SparklesIcon, UsersIcon } from "lucide-react";
import { Sidebar, SidebarContent, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarInset, SidebarMenu, SidebarMenuBadge, SidebarMenuButton, SidebarMenuItem, SidebarProvider, SidebarTrigger } from "frontend";

const NAV = [
    { badge: "231", icon: UsersIcon, label: "Operators" },
    { badge: null, icon: MapIcon, label: "Stages" },
    { badge: "6", icon: SparklesIcon, label: "Recruitment" },
    { badge: null, icon: BoxesIcon, label: "Depot" },
];

const OPERATORS = [
    { detail: "E2 60 · Guard", id: "char_4064_mlynar", name: "Mlynar", trust: "200%" },
    { detail: "E2 42 · Guard", id: "char_263_skadi", name: "Skadi the Corrupting Heart", trust: "164%" },
    { detail: "E2 55 · Vanguard", id: "char_102_texas", name: "Texas", trust: "189%" },
];

const PLANNER = [
    { icon: LayersIcon, label: "Upgrade planner" },
    { icon: HammerIcon, label: "Base layout" },
];

function Brand() {
    return (
        <div className="flex items-center gap-2 px-1 py-1.5">
            <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary font-semibold text-primary-foreground text-sm">M</div>
            <div className="grid flex-1 leading-tight">
                <span className="truncate font-semibold text-sm">Myrtle</span>
                <span className="truncate text-muted-foreground text-xs">Doctor Kal'tsit · EN</span>
            </div>
        </div>
    );
}

function Nav() {
    return (
        <div className="flex min-h-0 flex-1 flex-col">
            <SidebarContent>
            <SidebarGroup>
                <SidebarGroupLabel>Roster</SidebarGroupLabel>
                <SidebarGroupContent>
                    <SidebarMenu>
                        {NAV.map((item, i) => (
                            <SidebarMenuItem key={item.label}>
                                <SidebarMenuButton isActive={i === 0} tooltip={item.label}>
                                    <item.icon />
                                    <span>{item.label}</span>
                                </SidebarMenuButton>
                                {item.badge ? <SidebarMenuBadge>{item.badge}</SidebarMenuBadge> : null}
                            </SidebarMenuItem>
                        ))}
                    </SidebarMenu>
                </SidebarGroupContent>
            </SidebarGroup>
            <SidebarGroup>
                <SidebarGroupLabel>Planner</SidebarGroupLabel>
                <SidebarGroupContent>
                    <SidebarMenu>
                        {PLANNER.map((item) => (
                            <SidebarMenuItem key={item.label}>
                                <SidebarMenuButton tooltip={item.label}>
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
    );
}

function Workspace() {
    return (
        <SidebarInset>
            <header className="flex h-12 items-center gap-2 border-b px-4">
                <SidebarTrigger />
                <span className="font-medium text-sm">Operators</span>
                <span className="ml-auto text-muted-foreground text-xs tabular-nums">231 owned · 42 at E2</span>
            </header>
            <div className="grid gap-2 p-4">
                {OPERATORS.map((op) => (
                    <div className="flex items-center gap-3 rounded-lg border p-2" key={op.id}>
                        <img alt={op.name} className="size-10 shrink-0 rounded-lg object-cover" src={`https://api.myrtle.moe/api/avatar/${op.id}`} />
                        <div className="grid leading-tight">
                            <span className="font-medium text-sm">{op.name}</span>
                            <span className="text-muted-foreground text-xs">{op.detail}</span>
                        </div>
                        <span className="ml-auto text-muted-foreground text-xs tabular-nums">Trust {op.trust}</span>
                    </div>
                ))}
            </div>
        </SidebarInset>
    );
}

export const AppShell = () => (
    <SidebarProvider>
        <Sidebar collapsible="icon">
            <SidebarHeader>
                <Brand />
            </SidebarHeader>
            <Nav />
        </Sidebar>
        <Workspace />
    </SidebarProvider>
);

export const CollapsedToIcons = () => (
    <SidebarProvider defaultOpen={false}>
        <Sidebar collapsible="icon">
            <SidebarHeader>
                <Brand />
            </SidebarHeader>
            <Nav />
        </Sidebar>
        <Workspace />
    </SidebarProvider>
);

export const Floating = () => (
    <SidebarProvider className="bg-sidebar">
        <Sidebar collapsible="icon" variant="floating">
            <SidebarHeader>
                <Brand />
            </SidebarHeader>
            <Nav />
        </Sidebar>
        <Workspace />
    </SidebarProvider>
);

export const StaticPanel = () => (
    <SidebarProvider className="h-96 min-h-0 w-64 overflow-hidden rounded-xl border">
        <Sidebar collapsible="none">
            <SidebarHeader>
                <Brand />
            </SidebarHeader>
            <Nav />
        </Sidebar>
    </SidebarProvider>
);
