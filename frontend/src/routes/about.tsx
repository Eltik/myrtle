import { createFileRoute } from "@tanstack/react-router";
import { Kicker } from "#/components/ui/kicker";

export const Route = createFileRoute("/about")({
    component: About,
});

function About() {
    return (
        <main className="mx-auto w-[min(1080px,calc(100%-2rem))] px-4 py-12">
            <section className="rounded-2xl border border-border bg-card p-6 text-card-foreground shadow-[0_22px_44px_color-mix(in_srgb,var(--foreground)_10%,transparent),0_6px_18px_color-mix(in_srgb,var(--foreground)_8%,transparent)] backdrop-blur-[4px] sm:p-8">
                <Kicker className="mb-2">About</Kicker>
                <h1 className="mb-3 font-[Fraunces,Georgia,serif] text-4xl font-bold text-[var(--sea-ink)] sm:text-5xl">A small starter with room to grow.</h1>
                <p className="m-0 max-w-3xl text-base leading-8 text-[var(--sea-ink-soft)]">TanStack Start gives you type-safe routing, server functions, and modern SSR defaults. Use this as a clean foundation, then layer in your own routes, styling, and add-ons.</p>
            </section>
        </main>
    );
}
