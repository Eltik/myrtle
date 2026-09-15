import { Link } from "@tanstack/react-router";
import { Kicker } from "#/components/ui/kicker";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { featAccentVars } from "#/lib/role-styles";
import type { messages } from "./FeatureStrip.messages";
import styles from "./FeatureStrip.module.css";

type FeatureMessageKey = keyof typeof messages & string;

const FEATURES: { icon: string; kKey: FeatureMessageKey; tKey: FeatureMessageKey; dKey: FeatureMessageKey; accent: string; href: string }[] = [
    {
        icon: "database",
        kKey: "feature.data.kicker",
        tKey: "feature.data.title",
        dKey: "feature.data.desc",
        accent: "coral",
        href: "/operators",
    },
    {
        icon: "sync",
        kKey: "feature.sync.kicker",
        tKey: "feature.sync.title",
        dKey: "feature.sync.desc",
        accent: "mint",
        href: "/user/leaderboard",
    },
    {
        icon: "bolt",
        kKey: "feature.tools.kicker",
        tKey: "feature.tools.title",
        dKey: "feature.tools.desc",
        accent: "amber",
        href: "/tools/recruitment",
    },
];

function FeatIcon({ name }: { name: string }) {
    const p = { fill: "none", stroke: "currentColor", strokeWidth: 1.8, strokeLinecap: "round" as const, strokeLinejoin: "round" as const };
    switch (name) {
        case "database":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <ellipse cx="12" cy="5" rx="9" ry="3" />
                    <path d="M3 5v6c0 1.66 4 3 9 3s9-1.34 9-3V5" />
                    <path d="M3 11v6c0 1.66 4 3 9 3s9-1.34 9-3v-6" />
                </svg>
            );
        case "sync":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <path d="M21 12a9 9 0 0 0-15-6.7L3 8" />
                    <path d="M3 4v4h4" />
                    <path d="M3 12a9 9 0 0 0 15 6.7l3-2.7" />
                    <path d="M21 20v-4h-4" />
                </svg>
            );
        case "bolt":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <path d="M13 2 3 14h9l-1 8 10-12h-9l1-8z" />
                </svg>
            );
        default:
            return null;
    }
}

export default function FeatureStrip() {
    const t: TypedT<typeof messages> = useT("home");
    return (
        <section className="mx-auto grid w-[min(1080px,calc(100%-2rem))] grid-cols-1 gap-3.5 py-2 md:grid-cols-3">
            {FEATURES.map((f) => (
                <div className={styles.featCard} key={f.kKey} style={featAccentVars(f.accent)}>
                    <div className={styles.featIcon}>
                        <FeatIcon name={f.icon} />
                    </div>
                    <Kicker className="mb-0">{t(f.kKey)}</Kicker>
                    <div className="font-sans font-semibold text-[17px] text-foreground leading-snug tracking-tight">{t(f.tKey)}</div>
                    <p className="my-1 mb-3 font-sans text-[13.5px] text-muted-foreground leading-[1.55]">{t(f.dKey)}</p>
                    <Link to={f.href} className="group mt-auto inline-flex cursor-pointer items-center font-medium font-sans text-[12.5px] text-primary leading-none no-underline [&>svg]:h-3 [&>svg]:w-3">
                        {t("feature.explore")}{" "}
                        <svg className="ml-1.5 transition-[margin-left] duration-200 group-hover:ml-2.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" role="image" aria-label={t("feature.arrowIcon")}>
                            <path d="M5 12h14" />
                            <path d="m12 5 7 7-7 7" />
                        </svg>
                    </Link>
                </div>
            ))}
        </section>
    );
}
