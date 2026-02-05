"use client";

import { ArrowRight, Sparkles } from "lucide-react";
import Link from "next/link";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Button } from "~/components/ui/shadcn/button";
import { AnimatedGradientOrbs } from "./animated-gradient-orbs";

export function CTASection() {
    return (
        <section className="py-20 md:py-32">
            <div className="container mx-auto px-4">
                <InView
                    once
                    transition={{ duration: 0.6, ease: "easeOut" }}
                    variants={{
                        hidden: { opacity: 0, y: 30, scale: 0.95 },
                        visible: { opacity: 1, y: 0, scale: 1 },
                    }}
                >
                    <div className="relative overflow-hidden rounded-3xl border border-primary/20 bg-linear-to-br from-primary/10 via-primary/5 to-transparent p-12 md:p-16 lg:p-20">
                        {/* Animated background gradient orbs */}
                        <AnimatedGradientOrbs cursorInfluence={0.12} />

                        <div className="relative text-center">
                            <div className="mb-6 inline-flex items-center gap-2 rounded-full border border-primary/30 bg-primary/10 px-4 py-2 font-medium text-primary text-sm">
                                <Sparkles className="h-4 w-4" />
                                <span>Ready to optimize your gameplay?</span>
                            </div>

                            <h2 className="mb-6 text-balance font-bold text-3xl md:text-5xl">Start using Myrtle today</h2>

                            <p className="mx-auto mb-8 max-w-2xl text-balance text-lg text-muted-foreground md:text-xl">Join thousands of Doctors who have already enhanced their Arknights experience with our comprehensive toolkit.</p>

                            <div className="flex flex-col items-center justify-center gap-4 sm:flex-row">
                                <Button asChild className="group relative h-12 overflow-hidden rounded-lg bg-primary px-8 font-semibold text-lg text-primary-foreground shadow-[0_8px_24px_var(--glow-primary)] transition-all duration-300 hover:scale-102 hover:shadow-[0_12px_32px_var(--glow-primary)]" size="lg">
                                    <Link href="/collection/operators">
                                        <span className="pointer-events-none absolute inset-x-0 top-0 h-1/2 bg-linear-to-b from-white/25 to-transparent" />
                                        <span className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-linear-to-t from-black/20 to-transparent" />
                                        <span className="relative flex items-center">
                                            Get Started
                                            <ArrowRight className="ml-2 h-5 w-5 transition-transform group-hover:translate-x-1" />
                                        </span>
                                    </Link>
                                </Button>
                                <Button asChild className="h-12 rounded-lg border-2 border-primary/30 bg-secondary/50 px-8 font-semibold text-foreground text-lg transition-all duration-300 hover:border-primary/60 hover:bg-secondary" size="lg" variant="outline">
                                    <Link href="/tools/recruitment">View All Tools</Link>
                                </Button>
                            </div>
                        </div>
                    </div>
                </InView>
            </div>
        </section>
    );
}
