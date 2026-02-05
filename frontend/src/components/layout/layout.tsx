"use client";

import { Header } from "~/components/layout/header";
import { Footer } from "./footer";

interface LayoutProps {
    children: React.ReactNode;
}

export function Layout({ children }: LayoutProps) {
    return (
        <div className="relative flex min-h-screen overflow-x-clip bg-background">
            <div className="pointer-events-none absolute inset-0 overflow-hidden">
                <div className="absolute -top-40 -right-40 h-112 w-md rounded-full bg-primary/20 blur-120" />
                <div className="absolute top-1/2 -left-48 h-104 w-md rounded-full bg-primary/10 blur-100" />
                <div className="absolute right-1/3 -bottom-40 h-80 w-80 rounded-full bg-primary/15 blur-100" />
            </div>

            <div className="relative flex min-w-0 flex-1 flex-col">
                <Header />
                <main className="mx-auto mt-14 w-full min-w-0 max-w-6xl flex-1 px-3 py-8 sm:px-4 md:px-8 md:py-12">{children}</main>
                <Footer />
            </div>
        </div>
    );
}
