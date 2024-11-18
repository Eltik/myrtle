import { AppSidebar } from "./app-sidebar";
import { Navbar } from "./navbar";
import { SidebarProvider } from "~/components/ui/sidebar";

import { motion } from "framer-motion";
import type { ReactNode } from "react";
import { Toaster } from "../ui/toaster";

export default function Layout({ children }: { children: ReactNode }) {
    const variants = {
        hidden: { opacity: 0, x: -200, y: 0 },
        enter: { opacity: 1, x: 0, y: 0 },
        exit: { opacity: 0, x: 0, y: -100 },
    };

    return (
        <>
            <SidebarProvider defaultOpen={false}>
                <AppSidebar />
                <Navbar />
                <motion.main variants={variants} initial={"hidden"} animate={"enter"} exit={"exit"} transition={{ type: "linear" }} className="min-h-[calc(100svh_-_80px)] w-full transition-[margin-left] duration-300 ease-in-out lg:ml-[90px]">
                    <div className="fixed inset-0 -z-50 h-full w-full bg-[radial-gradient(#cccccc80_1px,transparent_1px)] [background-size:16px_16px] dark:bg-[radial-gradient(#fafafa12_1px,transparent_1px)]" />
                    <div className="mt-10" />
                    {children}
                    <Toaster />
                </motion.main>
            </SidebarProvider>
        </>
    );
}
