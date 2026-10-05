import type { ReactNode } from "react";

/** What a grid screen shows in place of the grid: a missing grid, or one the viewer may not edit. `action` is the way out. */
export function GridNotice({ title, body, action }: { title: string; body: string; action: ReactNode }) {
    return (
        <main className="mx-auto w-[min(720px,calc(100%-2rem))] py-20 text-center">
            <h1 className="m-0 font-bold font-sans text-2xl text-foreground tracking-tight">{title}</h1>
            <p className="mt-3 font-sans text-muted-foreground text-sm">{body}</p>
            {action}
        </main>
    );
}
