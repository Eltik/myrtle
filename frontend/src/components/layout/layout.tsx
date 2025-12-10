"use client";

import { Header } from "~/components/layout/header";
import { Footer } from "./footer";

interface LayoutProps {
    children: React.ReactNode;
}

export function Layout({ children }: LayoutProps) {
    return (
        <div className="relative flex min-h-screen overflow-x-hidden bg-background">
            <div className="pointer-events-none absolute inset-0 overflow-hidden">
                <div className="-top-40 -right-40 absolute h-96 w-96 rounded-full bg-primary/20 blur-[100px]" />
                <div className="-left-40 absolute top-1/2 h-80 w-80 rounded-full bg-primary/10 blur-[80px]" />
                <div className="-bottom-40 absolute right-1/3 h-72 w-72 rounded-full bg-primary/15 blur-[90px]" />
            </div>

            <div className="relative flex flex-1 flex-col">
                <Header />
                <main className="mx-auto mt-14 w-full min-w-0 max-w-6xl flex-1 px-4 py-8 md:px-8 md:py-12">{children}</main>
                <Footer />
            </div>
        </div>
    );
}
