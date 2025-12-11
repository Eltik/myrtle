import { AnimatePresence, motion } from "framer-motion";
import { Blocks, ChevronDown, Home, Search, Settings, ShieldHalf } from "lucide-react";
import Link from "next/link";
import { useEffect, useState } from "react";
import { Sidebar, SidebarContent, SidebarGroup, SidebarGroupContent, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarTrigger, useSidebar } from "~/components/ui/sidebar";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "../ui/collapsible";
import { Separator } from "../ui/separator";

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
                url: "/recruitment-calculator",
            },
            {
                title: "Chibi Viewer",
                url: "/chibis",
            },
            {
                title: "DPS Calculator",
                url: "/dps-calculator",
            },
            {
                title: "Squad Randomizer",
                url: "/squad-randomizer",
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
        <Sidebar className="z-40" collapsible="icon" variant="floating">
            <div className="-right-[16px] invisible absolute top-[12px] z-30 md:visible">
                <div className="rounded-md border bg-background p-1 shadow-lg transition-all duration-150 hover:bg-secondary">
                    <SidebarTrigger className="hover:bg-inherit" />
                </div>
            </div>
            <SidebarHeader className="border-b p-4">
                <Link className="flex items-center space-x-2 group-data-[collapsible=icon]:pointer-events-none group-data-[collapsible=icon]:absolute group-data-[collapsible=icon]:opacity-0" href="/">
                    <span className="pointer-events-auto font-bold text-lg opacity-100">myrtle.moe</span>
                </Link>
                <Link className="pointer-events-none hidden items-center space-x-2 opacity-0 group-data-[collapsible=icon]:pointer-events-auto group-data-[collapsible=icon]:flex group-data-[collapsible=icon]:opacity-100" href="/">
                    <span className="font-bold text-lg">M</span>
                </Link>
            </SidebarHeader>
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            {items.map((item, index) => (
                                <SidebarMenuItem key={index}>
                                    {item.group ? (
                                        <Collapsible className="group/collapsible" onOpenChange={() => toggleGroup(index)} open={openGroups[index]}>
                                            <CollapsibleTrigger asChild>
                                                <SidebarMenuButton className="border">
                                                    {item.icon && <item.icon className="mr-2 h-4 w-4" />}
                                                    {item.title}
                                                    <ChevronDown className={`ml-auto transition-transform ${open && openGroups[index] ? "rotate-180" : ""}`} />
                                                </SidebarMenuButton>
                                            </CollapsibleTrigger>
                                            <Separator className="mt-2 mb-1" />
                                            <AnimatePresence initial={false}>
                                                {openGroups[index] && (
                                                    <CollapsibleContent asChild forceMount>
                                                        <motion.div animate={{ height: "auto", opacity: 1 }} exit={{ height: 0, opacity: 0 }} initial={{ height: 0, opacity: 0 }} transition={{ duration: 0.3, ease: "easeInOut" }}>
                                                            <SidebarGroup>
                                                                <SidebarGroupContent>
                                                                    <SidebarMenu>
                                                                        {item.items?.map((subItem, subIndex) => (
                                                                            <motion.div animate={{ y: 0, opacity: 1 }} initial={{ y: -10, opacity: 0 }} key={`${index}-${subIndex}`} transition={{ delay: subIndex * 0.1 }}>
                                                                                <SidebarMenuItem>
                                                                                    <SidebarMenuButton asChild>
                                                                                        <Link className="relative pl-6" href={subItem.url}>
                                                                                            {subItem.title}
                                                                                        </Link>
                                                                                    </SidebarMenuButton>
                                                                                </SidebarMenuItem>
                                                                            </motion.div>
                                                                        ))}
                                                                    </SidebarMenu>
                                                                </SidebarGroupContent>
                                                            </SidebarGroup>
                                                            <div className="absolute top-12 bottom-0 left-2 w-0.5 rounded-md bg-border" />
                                                        </motion.div>
                                                    </CollapsibleContent>
                                                )}
                                            </AnimatePresence>
                                        </Collapsible>
                                    ) : (
                                        <SidebarMenuButton asChild>
                                            <Link className="border" href={item.url}>
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
