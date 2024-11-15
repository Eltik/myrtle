import { AppSidebar } from "./app-sidebar";
import { Navbar } from "./navbar";
import { SidebarProvider } from "./ui/sidebar";

import type { ReactNode } from "react";

export default function Layout({ children }: { children: ReactNode }) {
    return (
        <>
            <SidebarProvider>
                <AppSidebar />
                <Navbar />
                <main className="min-h-[calc(100svh_-_80px)] transition-[margin-left] duration-300 ease-in-out lg:ml-[90px]">
                    <div className="mt-10" />
                    {children}
                </main>
            </SidebarProvider>
        </>
    );
}
