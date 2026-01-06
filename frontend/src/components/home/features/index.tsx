"use client";
import Link from "next/link";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { FEATURES } from "./impl/constants";

export function FeaturesSection() {
    return (
        <section className="py-20 md:py-32" id="features">
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
                        <h2 className="mb-4 text-balance font-bold text-3xl md:text-5xl">Everything you need in one place</h2>
                        <p className="mx-auto max-w-2xl text-balance text-lg text-muted-foreground md:text-xl">Powerful tools and resources designed to elevate your Arknights gameplay experience.</p>
                    </div>
                </InView>

                <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
                    {FEATURES.map((feature, index) => {
                        const Icon = feature.icon;
                        return (
                            <InView
                                key={feature.title}
                                once
                                transition={{
                                    duration: 0.5,
                                    ease: "easeOut",
                                    delay: index * 0.1,
                                }}
                                variants={{
                                    hidden: { opacity: 0, y: 30, scale: 0.95 },
                                    visible: { opacity: 1, y: 0, scale: 1 },
                                }}
                            >
                                <Link className="group block h-full" href={feature.href}>
                                    <Card className="h-full border-border/50 transition-all duration-300 hover:scale-[1.02] hover:border-primary/50 hover:shadow-lg hover:shadow-primary/10">
                                        <CardHeader>
                                            <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10 transition-colors duration-300 group-hover:bg-primary/20">
                                                <Icon className="h-6 w-6 text-primary" />
                                            </div>
                                            <CardTitle className="text-xl">{feature.title}</CardTitle>
                                        </CardHeader>
                                        <CardContent>
                                            <CardDescription className="text-base leading-relaxed">{feature.description}</CardDescription>
                                        </CardContent>
                                    </Card>
                                </Link>
                            </InView>
                        );
                    })}
                </div>
            </div>
        </section>
    );
}
