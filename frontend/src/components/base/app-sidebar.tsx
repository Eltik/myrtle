import { Blocks, ChevronDown, Home, Search, Settings, ShieldHalf } from "lucide-react";
import Link from "next/link";

import { Sidebar, SidebarContent, SidebarGroup, SidebarGroupContent, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarTrigger, useSidebar } from "~/components/ui/sidebar";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "../ui/collapsible";
import { Separator } from "../ui/separator";
import { useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";

const items = [
    {
        title: "Home",
        url: "/",
        pathNames: ["/"],
        icon: Home,
    },
    {
        title: "Search",
        url: "#",
        pathNames: ["/search"],
        icon: Search,
    },
    {
        title: "Tools",
        url: "#",
        icon: Blocks,
        group: true,
        pathNames: ["/user", "/operators", "/recruitment", "/squad"],
        items: [
            {
                title: "Operators",
                url: "/operators",
            },
            {
                title: "Recruitment Calculator",
                url: "#",
            },
            {
                title: "DPS Calculator",
                url: "/dps-calculator",
            },
            {
                title: "Squad Randomizer",
                url: "#",
            },
        ],
    },
    {
        title: "Leaderboard",
        url: "/leaderboard",
        pathNames: ["/leaderboard"],
        icon: ShieldHalf,
    },
    {
        title: "Settings",
        url: "#",
        pathNames: ["/settings"],
        icon: Settings,
    },
];

export function AppSidebar() {
    const [openGroups, setOpenGroups] = useState<Record<string, boolean>>({});
    const { open, setOpen } = useSidebar();

    const toggleGroup = (index: number) => {
        setOpenGroups((prev) => ({ ...prev, [index]: !prev[index] }));
        if (!open) {
            setOpen(true);
        }
    };

    useEffect(() => {
        if (!open) {
            setOpenGroups({});
        }
    }, [open]);

    return (
        <Sidebar variant="floating" collapsible="icon" className="z-40">
            <div className="invisible absolute -right-[16px] top-[12px] z-30 md:visible">
                <div className="rounded-md border bg-background p-1 shadow-lg transition-all duration-150 hover:bg-secondary">
                    <SidebarTrigger className="hover:bg-inherit" />
                </div>
            </div>
            <SidebarHeader className="border-b p-4">
                <Link href="/" className="flex items-center space-x-2 group-data-[collapsible=icon]:pointer-events-none group-data-[collapsible=icon]:absolute group-data-[collapsible=icon]:opacity-0">
                    <span className="pointer-events-auto text-lg font-bold opacity-100">myrtle.moe</span>
                </Link>
                <Link href="/" className="pointer-events-none hidden items-center space-x-2 opacity-0 group-data-[collapsible=icon]:pointer-events-auto group-data-[collapsible=icon]:flex group-data-[collapsible=icon]:opacity-100">
                    <span className="text-lg font-bold">M</span>
                </Link>
            </SidebarHeader>
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            {items.map((item, index) => (
                                <SidebarMenuItem key={index}>
                                    {item.group ? (
                                        <Collapsible className="group/collapsible" open={openGroups[index]} onOpenChange={() => toggleGroup(index)}>
                                            <CollapsibleTrigger asChild>
                                                <SidebarMenuButton className="border">
                                                    {item.icon && <item.icon className="mr-2 h-4 w-4" />}
                                                    {item.title}
                                                    <ChevronDown className={`ml-auto transition-transform ${open && openGroups[index] ? "rotate-180" : ""}`} />
                                                </SidebarMenuButton>
                                            </CollapsibleTrigger>
                                            <Separator className="mb-1 mt-2" />
                                            <AnimatePresence initial={false}>
                                                {openGroups[index] && (
                                                    <CollapsibleContent forceMount asChild>
                                                        <motion.div initial={{ height: 0, opacity: 0 }} animate={{ height: "auto", opacity: 1 }} exit={{ height: 0, opacity: 0 }} transition={{ duration: 0.3, ease: "easeInOut" }}>
                                                            <SidebarGroup>
                                                                <SidebarGroupContent>
                                                                    <SidebarMenu>
                                                                        {item.items?.map((subItem, subIndex) => (
                                                                            <motion.div key={`${index}-${subIndex}`} initial={{ y: -10, opacity: 0 }} animate={{ y: 0, opacity: 1 }} transition={{ delay: subIndex * 0.1 }}>
                                                                                <SidebarMenuItem>
                                                                                    <SidebarMenuButton asChild>
                                                                                        <Link href={subItem.url} className="relative pl-6">
                                                                                            {subItem.title}
                                                                                        </Link>
                                                                                    </SidebarMenuButton>
                                                                                </SidebarMenuItem>
                                                                            </motion.div>
                                                                        ))}
                                                                    </SidebarMenu>
                                                                </SidebarGroupContent>
                                                            </SidebarGroup>
                                                            <div className="absolute bottom-0 left-2 top-12 w-0.5 rounded-md bg-border" />
                                                        </motion.div>
                                                    </CollapsibleContent>
                                                )}
                                            </AnimatePresence>
                                        </Collapsible>
                                    ) : (
                                        <SidebarMenuButton asChild>
                                            <Link href={item.url} className="border">
                                                {item.icon && <item.icon className="mr-2 h-4 w-4" />}
                                                {item.title}
                                            </Link>
                                        </SidebarMenuButton>
                                    )}
                                </SidebarMenuItem>
                            ))}
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
        </Sidebar>
    );
}
