"use client";

import { CTASection } from "~/components/home/cta";
import { FeaturesSection } from "~/components/home/features";
import { HeroSection } from "~/components/home/hero";
import { HowItWorksSection } from "~/components/home/how-it-works";
import { StatsSection } from "~/components/home/stats";

export default function HomePage() {
    return (
        <div className="relative w-full">
            <HeroSection />
            <FeaturesSection />
            <HowItWorksSection />
            <StatsSection />
            <CTASection />
        </div>
    );
}
