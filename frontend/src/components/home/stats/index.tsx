"use client";
import { useState } from "react";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { STATS } from "./impl/constants";

function StatCard({ stat, index }: { stat: (typeof STATS)[number]; index: number }) {
    const [isInView, setIsInView] = useState(false);
    const decimals = "decimals" in stat ? stat.decimals : 0;

    return (
        <InView
            once
            onInView={() => setIsInView(true)}
            transition={{
                duration: 0.5,
                ease: "easeOut",
                delay: index * 0.1,
            }}
            variants={{
                hidden: { opacity: 0, scale: 0.8 },
                visible: { opacity: 1, scale: 1 },
            }}
        >
            <div className="group relative overflow-hidden rounded-2xl border border-border/50 bg-card p-8 text-center transition-all duration-300 hover:border-primary/50 hover:shadow-lg hover:shadow-primary/10">
                <div className="absolute inset-0 bg-linear-to-br from-primary/5 via-transparent to-transparent opacity-0 transition-opacity duration-300 group-hover:opacity-100" />
                <div className="relative">
                    <div className="mb-2 font-bold text-4xl text-primary md:text-5xl">
                        <AnimatedNumber decimals={decimals} springOptions={{ bounce: 0, duration: 2000 }} value={isInView ? stat.value : 0} />
                        {stat.suffix}
                    </div>
                    <div className="mb-1 font-semibold text-foreground text-lg">{stat.label}</div>
                    <div className="text-muted-foreground text-sm">{stat.description}</div>
                </div>
            </div>
        </InView>
    );
}

export function StatsSection() {
    return (
        <section className="py-20 md:py-32">
            <div className="container mx-auto px-4">
                <InView
                    once
                    transition={{ duration: 0.6, ease: "easeOut" }}
                    variants={{
                        hidden: { opacity: 0, y: 20 },
                        visible: { opacity: 1, y: 0 },
                    }}
                >
                    <div className="mb-16 text-center">
                        <h2 className="mb-4 text-balance font-bold text-3xl md:text-5xl">Trusted by Doctors worldwide</h2>
                        <p className="mx-auto max-w-2xl text-balance text-lg text-muted-foreground md:text-xl">Join thousands of players who rely on our tools daily.</p>
                    </div>
                </InView>

                <div className="mx-auto max-w-5xl">
                    <div className="grid gap-8 sm:grid-cols-2 lg:grid-cols-4">
                        {STATS.map((stat, index) => (
                            <StatCard index={index} key={stat.label} stat={stat} />
                        ))}
                    </div>
                </div>
            </div>
        </section>
    );
}
