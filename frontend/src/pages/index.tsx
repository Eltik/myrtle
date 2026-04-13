"use client";

import { CTASection } from "~/components/home/cta";
import { FeaturesSection } from "~/components/home/features";
import { HeroSection } from "~/components/home/hero";
import { HowItWorksSection } from "~/components/home/how-it-works";
import { StatsSection } from "~/components/home/stats";
import { SEO } from "~/components/seo";

export default function HomePage() {
    return (
        <>
            <SEO
                description="Your comprehensive Arknights companion - DPS calculator with 100% accuracy, operator database, tier lists, recruitment calculator, account sync via Yostar OAuth, and community leaderboards."
                keywords={["Arknights companion", "DPS calculator", "tier list", "recruitment calculator", "account sync", "leaderboard"]}
                path="/"
                title="Arknights Companion"
            />
            <div className="relative w-full">
                <HeroSection />
                <FeaturesSection />
                <HowItWorksSection />
                <StatsSection />
                <CTASection />
            </div>
        </>
    );
}
