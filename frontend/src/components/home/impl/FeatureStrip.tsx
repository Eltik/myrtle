import { Kicker } from "#/components/ui/kicker";
import { featAccentVars } from "#/lib/role-styles";

const FEATURES = [
    {
        icon: "database",
        k: "Live data",
        t: "400+ operators, instant search",
        d: "Skills, talents, modules, skins, voice lines. Filter by faction, archetype, or tag.",
        accent: "coral",
    },
    {
        icon: "sync",
        k: "Roster sync",
        t: "Your account, mirrored",
        d: "Link a Yostar account and see your live box, E2 progress, base layout in real time.",
        accent: "mint",
    },
    {
        icon: "bolt",
        k: "Tools",
        t: "DPS, recruit, randomizer",
        d: "Interactive calculators and community-maintained tier lists for every stage meta.",
        accent: "amber",
    },
];

function FeatIcon({ name }: { name: string }) {
    const p = { fill: "none", stroke: "currentColor", strokeWidth: 1.8, strokeLinecap: "round" as const, strokeLinejoin: "round" as const };
    switch (name) {
        case "database":
            return (
                <svg viewBox="0 0 24 24" {...p}>
                    <ellipse cx="12" cy="5" rx="9" ry="3" />
                    <path d="M3 5v6c0 1.66 4 3 9 3s9-1.34 9-3V5" />
                    <path d="M3 11v6c0 1.66 4 3 9 3s9-1.34 9-3v-6" />
                </svg>
            );
        case "sync":
            return (
                <svg viewBox="0 0 24 24" {...p}>
                    <path d="M21 12a9 9 0 0 0-15-6.7L3 8" />
                    <path d="M3 4v4h4" />
                    <path d="M3 12a9 9 0 0 0 15 6.7l3-2.7" />
                    <path d="M21 20v-4h-4" />
                </svg>
            );
        case "bolt":
            return (
                <svg viewBox="0 0 24 24" {...p}>
                    <path d="M13 2 3 14h9l-1 8 10-12h-9l1-8z" />
                </svg>
            );
        default:
            return null;
    }
}

export default function FeatureStrip() {
    return (
        <section className="page-wrap grid grid-cols-1 gap-3.5 py-2 md:grid-cols-3">
            {FEATURES.map((f, i) => (
                <div className="feat-card" key={i} style={featAccentVars(f.accent)}>
                    <div className="feat-icon">
                        <FeatIcon name={f.icon} />
                    </div>
                    <Kicker className="mb-0">{f.k}</Kicker>
                    <div className="font-sans text-[17px] font-semibold leading-snug tracking-tight text-foreground">{f.t}</div>
                    <p className="my-1 mb-3 font-sans text-[13.5px] leading-[1.55] text-muted-foreground">{f.d}</p>
                    <a className="mt-auto inline-flex cursor-pointer items-center gap-1.5 font-sans text-[12.5px] font-medium leading-none text-primary no-underline transition-[gap] hover:gap-2.5 [&>svg]:h-3 [&>svg]:w-3" href="#">
                        Explore{" "}
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M5 12h14" />
                            <path d="m12 5 7 7-7 7" />
                        </svg>
                    </a>
                </div>
            ))}
        </section>
    );
}
