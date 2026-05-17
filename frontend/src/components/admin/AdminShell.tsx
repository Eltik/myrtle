import { useState } from "react";
import { AdminSidebar } from "./AdminSidebar";
import { AdminTopBar, type IAdminCrumb } from "./AdminTopBar";

interface IAdminShellProps {
    crumbs: IAdminCrumb[];
    children: React.ReactNode;
}

export function AdminShell({ crumbs, children }: IAdminShellProps): React.ReactElement {
    const [sidebarOpen, setSidebarOpen] = useState(false);
    return (
        <div className="flex min-h-screen bg-background text-foreground">
            <AdminSidebar open={sidebarOpen} onClose={() => setSidebarOpen(false)} />
            <main className="flex min-w-0 flex-1 flex-col">
                <AdminTopBar crumbs={crumbs} onOpenSidebar={() => setSidebarOpen(true)} />
                <div className="w-full max-w-330 px-4 pt-4 pb-10 sm:px-6 sm:pt-5 lg:px-8 lg:pt-6 lg:pb-12">{children}</div>
            </main>
        </div>
    );
}

export function PageHead({ kicker, title, sub, action }: { kicker: string; title: string; sub?: React.ReactNode; action?: React.ReactNode }): React.ReactElement {
    return (
        <div className="mb-5 flex flex-col gap-3 border-border border-b pb-4 sm:flex-row sm:items-end sm:justify-between sm:gap-4">
            <div className="flex min-w-0 flex-col gap-1.5">
                <span className="font-bold text-[11px] text-primary uppercase tracking-[0.22em]">{kicker}</span>
                <h1 className="font-semibold text-[22px] leading-tight tracking-[-0.02em] sm:text-[26px]">{title}</h1>
                {sub ? <p className="max-w-[64ch] text-[13px] text-muted-foreground leading-[1.55] sm:text-[13.5px]">{sub}</p> : null}
            </div>
            {action ? <div className="flex flex-wrap items-center gap-2">{action}</div> : null}
        </div>
    );
}

export function HCode({ children }: { children: React.ReactNode }): React.ReactElement {
    return <code className="rounded-[7px] border border-border bg-muted px-1.5 py-0.5 font-mono text-[0.9em]">{children}</code>;
}
