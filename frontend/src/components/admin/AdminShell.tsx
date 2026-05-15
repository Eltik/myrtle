import type * as React from "react";
import { AdminSidebar } from "./AdminSidebar";
import { AdminTopBar } from "./AdminTopBar";

interface IAdminShellProps {
    crumbs: string[];
    children: React.ReactNode;
}

export function AdminShell({ crumbs, children }: IAdminShellProps): React.ReactElement {
    return (
        <div className="grid min-h-screen grid-cols-[256px_1fr] bg-background text-foreground">
            <AdminSidebar />
            <main className="flex min-w-0 flex-col">
                <AdminTopBar crumbs={crumbs} />
                <div className="w-full max-w-[1320px] px-8 pt-6 pb-12">{children}</div>
            </main>
        </div>
    );
}

export function PageHead({ kicker, title, sub, action }: { kicker: string; title: string; sub?: React.ReactNode; action?: React.ReactNode }): React.ReactElement {
    return (
        <div className="mb-5 flex items-end justify-between gap-4 border-b border-border pb-4">
            <div className="flex flex-col gap-1.5">
                <span className="font-bold text-[11px] uppercase tracking-[0.22em] text-primary">{kicker}</span>
                <h1 className="font-semibold text-[26px] leading-tight tracking-[-0.02em]">{title}</h1>
                {sub ? <p className="max-w-[64ch] text-[13.5px] leading-[1.55] text-muted-foreground">{sub}</p> : null}
            </div>
            {action ? <div className="flex items-center gap-2">{action}</div> : null}
        </div>
    );
}

export function HCode({ children }: { children: React.ReactNode }): React.ReactElement {
    return <code className="rounded-[7px] border border-border bg-muted px-1.5 py-0.5 font-mono text-[0.9em]">{children}</code>;
}
