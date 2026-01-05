"use client";

import { ArrowRight, Sparkles } from "lucide-react";
import Link from "next/link";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Button } from "~/components/ui/shadcn/button";

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
                        {/* Background decoration */}
                        <div className="pointer-events-none absolute inset-0">
                            <div className="-right-20 -top-20 absolute h-64 w-64 rounded-full bg-primary/20 blur-3xl" />
                            <div className="-bottom-20 -left-20 absolute h-64 w-64 rounded-full bg-primary/15 blur-3xl" />
                        </div>

                        <div className="relative text-center">
                            <div className="mb-6 inline-flex items-center gap-2 rounded-full border border-primary/30 bg-primary/10 px-4 py-2 font-medium text-primary text-sm">
                                <Sparkles className="h-4 w-4" />
                                <span>Ready to optimize your gameplay?</span>
                            </div>

                            <h2 className="mb-6 text-balance font-bold text-3xl md:text-5xl">Start using Myrtle today</h2>

                            <p className="mx-auto mb-8 max-w-2xl text-balance text-lg text-muted-foreground md:text-xl">Join thousands of Doctors who have already enhanced their Arknights experience with our comprehensive toolkit.</p>

                            <div className="flex flex-col items-center justify-center gap-4 sm:flex-row">
                                <Button asChild className="group h-12 px-8 text-lg" size="lg">
                                    <Link href="/operators">
                                        Get Started Free
                                        <ArrowRight className="ml-2 h-5 w-5 transition-transform group-hover:translate-x-1" />
                                    </Link>
                                </Button>
                                <Button asChild className="h-12 bg-transparent px-8 text-lg" size="lg" variant="outline">
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
