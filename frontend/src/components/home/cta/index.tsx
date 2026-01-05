"use client";

import Link from "next/link";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent } from "~/components/ui/shadcn/card";

const ANIMATION_VARIANTS = {
    hidden: { opacity: 0, scale: 0.95 },
    visible: { opacity: 1, scale: 1 },
} as const;

export function CTASection() {
    return (
        <section className="px-4 py-20">
            <InView transition={{ duration: 0.6 }} variants={ANIMATION_VARIANTS}>
                <Card className="mx-auto max-w-4xl border-2 border-primary/30 bg-linear-to-br from-primary/5 to-background">
                    <CardContent className="p-12 text-center">
                        <h2 className="mb-4 font-bold text-3xl md:text-4xl">Ready to Elevate Your Game?</h2>
                        <p className="mx-auto mb-8 max-w-2xl text-lg text-muted-foreground">Create a free account to unlock profile tracking, save your favorite operators, and join the community.</p>
                        <div className="flex flex-col justify-center gap-4 sm:flex-row">
                            <Button className="h-12 px-8 text-lg" size="lg">
                                <Link href="/auth/signup">Sign Up Free</Link>
                            </Button>
                            <Button className="h-12 bg-transparent px-8 text-lg" size="lg" variant="outline">
                                <Link href="/operators">Explore Without Account</Link>
                            </Button>
                        </div>
                    </CardContent>
                </Card>
            </InView>
        </section>
    );
}
