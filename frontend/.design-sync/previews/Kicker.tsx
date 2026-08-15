import { Kicker } from "frontend";

export const SectionHeading = () => (
    <div>
        <Kicker>Community</Kicker>
        <h2 className="m-0 mb-1.5 font-bold font-sans text-4xl text-foreground leading-tight tracking-tight">Tier lists, at a glance.</h2>
        <p className="mt-1.5 max-w-130 font-sans text-muted-foreground text-sm leading-normal">Previews from the most-watched community lists. Click any card to open the full ranking.</p>
    </div>
);

export const PanelHeader = () => (
    <section className="w-full max-w-xl overflow-hidden rounded-[14px] border border-border bg-card">
        <div className="flex flex-wrap items-center gap-2.5 border-border border-b px-4 py-3">
            <Kicker className="mb-0">Spawn Schedule</Kicker>
            <span className="font-mono text-[11px] text-muted-foreground tabular-nums">9 groups</span>
            <span className="h-px flex-1 bg-border" />
        </div>
        <ul className="m-0 flex list-none flex-col gap-2 p-4 font-sans text-muted-foreground text-sm">
            <li className="flex items-center justify-between gap-4">
                <span className="text-foreground">Originium Slug</span>
                <span className="font-mono text-xs tabular-nums">×6 · 0:12</span>
            </li>
            <li className="flex items-center justify-between gap-4">
                <span className="text-foreground">Armed Sarkaz Shieldbearer</span>
                <span className="font-mono text-xs tabular-nums">×2 · 0:38</span>
            </li>
            <li className="flex items-center justify-between gap-4">
                <span className="text-foreground">Winged Mistcaller</span>
                <span className="font-mono text-xs tabular-nums">×1 · 1:24</span>
            </li>
        </ul>
    </section>
);

export const CardLabels = () => (
    <div className="grid w-full max-w-2xl gap-4 sm:grid-cols-2">
        <div className="rounded-lg border border-border bg-card px-4 py-3">
            <Kicker className="mb-0">Today</Kicker>
            <p className="m-0 mt-1 font-semibold font-sans text-foreground text-xl">2 operator birthdays</p>
            <p className="m-0 mt-1 font-sans text-muted-foreground text-sm">Texas and Exusiai are celebrating.</p>
        </div>
        <div className="rounded-lg border border-border bg-card px-4 py-3">
            <Kicker className="mb-0">Account</Kicker>
            <p className="m-0 mt-1 font-semibold font-sans text-foreground text-xl">Roster synced</p>
            <p className="m-0 mt-1 font-sans text-muted-foreground text-sm">231 operators imported 4 hours ago.</p>
        </div>
    </div>
);
