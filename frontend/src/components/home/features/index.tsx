"use client";

import { InView } from "~/components/ui/motion-primitives/in-view";
import { ANIMATION_VARIANTS, FEATURES } from "./impl/constants";
import { FeatureCard } from "./impl/feature-card";

export function FeaturesSection() {
    return (
        <section className="px-4 py-20">
            <InView once transition={{ duration: 0.6 }} variants={ANIMATION_VARIANTS.header}>
                <div className="mb-12 text-center">
                    <h2 className="mb-4 font-bold text-3xl md:text-4xl">Try It Now, No Account Required</h2>
                    <p className="mx-auto max-w-2xl text-lg text-muted-foreground">Jump right in and explore our powerful tools designed for every Doctor</p>
                </div>
            </InView>

            <div className="mx-auto grid max-w-7xl grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
                {FEATURES.map((feature, index) => (
                    <FeatureCard feature={feature} index={index} key={feature.title} />
                ))}
            </div>
        </section>
    );
}
