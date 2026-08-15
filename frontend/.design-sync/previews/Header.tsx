import { Header } from "frontend";

// The site-wide sticky top bar: logo + version chip, the desktop nav row, the
// ⌘K search affordance, theme/donate/GitHub actions and the account chip. It
// takes no props - the nav model, the session and the command palette all come
// from hooks - so the stories vary the page around it instead.
//
// The capture viewport is 900px wide, i.e. below the `lg` breakpoint, so the bar
// renders its tablet layout: hamburger instead of the `MainNav` link row.

export const Default = () => <Header />;

export const OverPageContent = () => (
    <div className="w-full">
        <Header />
        <main className="mx-auto w-[min(1080px,calc(100%-2rem))] py-10">
            <p className="m-0 mb-2 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-widest">Collection</p>
            <h1 className="m-0 mb-3 font-bold font-sans text-4xl text-foreground leading-tight tracking-tight">Operators</h1>
            <p className="m-0 mb-8 max-w-[52ch] font-sans text-[15px] text-muted-foreground leading-relaxed">438 operators with full stats, skills, talents and modules. The header stays pinned as the roster scrolls beneath it.</p>
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 md:grid-cols-3">
                {["Mlynar — 6★ Guard", "Skadi — 6★ Guard", "Texas — 5★ Vanguard", "Eyjafjalla — 6★ Caster", "Muelsyse — 6★ Vanguard", "Myrtle — 4★ Vanguard"].map((row) => (
                    <div key={row} className="rounded-lg border border-border bg-card px-4 py-3 font-sans text-[13.5px] text-foreground">
                        {row}
                    </div>
                ))}
            </div>
        </main>
    </div>
);
