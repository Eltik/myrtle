"use client";

import { CTASection } from "~/components/home/cta";
import { FeaturesSection } from "~/components/home/features";
import { HeroSection } from "~/components/home/hero";

export default function HomePage() {
    return (
        <div className="relative w-full">
            <HeroSection />
            <FeaturesSection />
            <CTASection />
        </div>
    );
}
