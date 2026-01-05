"use client";

import { ArrowRight, BookOpen, Calculator, Clock, Database, Shield, Sparkles, Users } from "lucide-react";
import Image from "next/image";
import Link from "next/link";
import { Marquee } from "~/components/ui/marquee";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { TextLoop } from "~/components/ui/motion-primitives/text-loop";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent } from "~/components/ui/shadcn/card";

export default function HomePage() {
    return (
        <div className="relative w-full">
            {/* Hero Section - Full width breakout */}
            <section className="hero-full-width relative flex min-h-[80vh] flex-col items-center justify-center overflow-hidden py-20">
                {/* Parallax Background Images - CSS-only animation for performance */}
                <div className="absolute inset-0 z-[1] flex flex-col justify-center">
                    <Marquee className="mb-4" duration={60}>
                        <Image alt="" className="hero-image rounded-lg object-cover" height={300} loading="eager" priority src="/images/cg3.png" width={400} />
                        <Image alt="" className="hero-image rounded-lg object-cover" height={300} loading="eager" priority src="/images/cg1.png" width={400} />
                        <Image alt="" className="hero-image rounded-lg object-cover" height={300} loading="eager" priority src="/images/cg2.png" width={400} />
                    </Marquee>
                    <Marquee className="mt-4" duration={80} reverse>
                        <Image alt="" className="hero-image-secondary rounded-lg object-cover" height={300} loading="eager" src="/images/cg2.png" width={400} />
                        <Image alt="" className="hero-image-secondary rounded-lg object-cover" height={300} loading="eager" src="/images/cg3.png" width={400} />
                        <Image alt="" className="hero-image-secondary rounded-lg object-cover" height={300} loading="eager" src="/images/cg1.png" width={400} />
                    </Marquee>
                </div>

                {/* Hero Content */}
                <div className="relative z-10 mx-auto max-w-4xl px-4 text-center">
                    <InView
                        transition={{ duration: 0.6, ease: "easeOut" }}
                        variants={{
                            hidden: { opacity: 0, y: 30, scale: 0.95, filter: "blur(10px)" },
                            visible: { opacity: 1, y: 0, scale: 1, filter: "blur(0px)" },
                        }}
                    >
                        <h1 className="mb-6 text-balance font-bold text-5xl leading-tight md:text-7xl">
                            Sync, discover, and optimize your{" "}
                            <TextLoop
                                className="text-primary"
                                interval={2.5}
                                style={{
                                    textShadow: "0 0 20px var(--glow-text-strong), 0 0 40px var(--glow-text-medium), 0 0 60px var(--glow-text-soft)",
                                }}
                            >
                                <span>operators</span>
                                <span>roster</span>
                                <span>recruitment</span>
                                <span>profile</span>
                                <span>progress</span>
                            </TextLoop>
                        </h1>
                    </InView>

                    <InView
                        transition={{ duration: 0.6, ease: "easeOut", delay: 0.2 }}
                        variants={{
                            hidden: { opacity: 0, y: 20 },
                            visible: { opacity: 1, y: 0 },
                        }}
                    >
                        <p className="mx-auto mb-8 max-w-2xl text-balance text-muted-foreground text-xl md:text-2xl">The ultimate toolkit for Arknights Doctors. Track operators, optimize teams, and master recruitment with powerful analytics.</p>
                    </InView>

                    <InView
                        transition={{ duration: 0.6, ease: "easeOut", delay: 0.4 }}
                        variants={{
                            hidden: { opacity: 0, y: 20 },
                            visible: { opacity: 1, y: 0 },
                        }}
                    >
                        <div className="flex flex-col items-center justify-center gap-4 sm:flex-row">
                            <Button asChild className="group h-12 px-8 text-lg" size="lg">
                                <Link href="/operators">
                                    Browse Operators
                                    <ArrowRight className="ml-2 h-5 w-5 transition-transform group-hover:translate-x-1" />
                                </Link>
                            </Button>
                            <Button asChild className="h-12 bg-transparent px-8 text-lg" size="lg" variant="outline">
                                <Link href="/tools/recruitment">
                                    <BookOpen className="mr-2 h-5 w-5" />
                                    Getting Started Guide
                                </Link>
                            </Button>
                        </div>
                    </InView>
                </div>
            </section>

            {/* Feature Showcase - No Account Needed */}
            <section className="px-4 py-20">
                <InView
                    transition={{ duration: 0.6 }}
                    variants={{
                        hidden: { opacity: 0, y: 30 },
                        visible: { opacity: 1, y: 0 },
                    }}
                >
                    <div className="mb-12 text-center">
                        <h2 className="mb-4 font-bold text-3xl md:text-4xl">Try It Now, No Account Required</h2>
                        <p className="mx-auto max-w-2xl text-lg text-muted-foreground">Jump right in and explore our powerful tools designed for every Doctor</p>
                    </div>
                </InView>

                <div className="mx-auto grid max-w-7xl grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
                    <InView
                        transition={{ duration: 0.5, delay: 0.1 }}
                        variants={{
                            hidden: { opacity: 0, y: 30, scale: 0.95 },
                            visible: { opacity: 1, y: 0, scale: 1 },
                        }}
                    >
                        <Link href="/operators">
                            <Card className="group h-full cursor-pointer border-2 transition-all duration-300 hover:scale-[1.02] hover:border-primary/50 hover:shadow-xl">
                                <CardContent className="p-6">
                                    <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10 transition-colors group-hover:bg-primary/20">
                                        <Database className="h-6 w-6 text-primary" />
                                    </div>
                                    <h3 className="mb-2 font-semibold text-xl">Operator Database</h3>
                                    <p className="text-muted-foreground">Browse detailed stats, skills, and builds for every operator in Arknights.</p>
                                </CardContent>
                            </Card>
                        </Link>
                    </InView>

                    <InView
                        transition={{ duration: 0.5, delay: 0.2 }}
                        variants={{
                            hidden: { opacity: 0, y: 30, scale: 0.95 },
                            visible: { opacity: 1, y: 0, scale: 1 },
                        }}
                    >
                        <Link href="/tools/recruitment">
                            <Card className="group h-full cursor-pointer border-2 transition-all duration-300 hover:scale-[1.02] hover:border-primary/50 hover:shadow-xl">
                                <CardContent className="p-6">
                                    <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10 transition-colors group-hover:bg-primary/20">
                                        <Calculator className="h-6 w-6 text-primary" />
                                    </div>
                                    <h3 className="mb-2 font-semibold text-xl">Recruitment Calculator</h3>
                                    <p className="text-muted-foreground">Find the perfect operator combinations with our advanced tag calculator.</p>
                                </CardContent>
                            </Card>
                        </Link>
                    </InView>

                    <InView
                        transition={{ duration: 0.5, delay: 0.3 }}
                        variants={{
                            hidden: { opacity: 0, y: 30, scale: 0.95 },
                            visible: { opacity: 1, y: 0, scale: 1 },
                        }}
                    >
                        <Link href="/operators/tier-list">
                            <Card className="group h-full cursor-pointer border-2 transition-all duration-300 hover:scale-[1.02] hover:border-primary/50 hover:shadow-xl">
                                <CardContent className="p-6">
                                    <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10 transition-colors group-hover:bg-primary/20">
                                        <Sparkles className="h-6 w-6 text-primary" />
                                    </div>
                                    <h3 className="mb-2 font-semibold text-xl">Tier Lists</h3>
                                    <p className="text-muted-foreground">Compare operators with community-driven tier lists and rankings.</p>
                                </CardContent>
                            </Card>
                        </Link>
                    </InView>

                    <InView
                        transition={{ duration: 0.5, delay: 0.4 }}
                        variants={{
                            hidden: { opacity: 0, y: 30, scale: 0.95 },
                            visible: { opacity: 1, y: 0, scale: 1 },
                        }}
                    >
                        <Card className="group h-full border-2 transition-all duration-300 hover:scale-[1.02] hover:border-primary/50 hover:shadow-xl">
                            <CardContent className="p-6">
                                <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10 transition-colors group-hover:bg-primary/20">
                                    <Clock className="h-6 w-6 text-primary" />
                                </div>
                                <h3 className="mb-2 font-semibold text-xl">Event Timeline</h3>
                                <p className="text-muted-foreground">Stay updated on current and upcoming events, banners, and schedules.</p>
                            </CardContent>
                        </Card>
                    </InView>

                    <InView
                        transition={{ duration: 0.5, delay: 0.5 }}
                        variants={{
                            hidden: { opacity: 0, y: 30, scale: 0.95 },
                            visible: { opacity: 1, y: 0, scale: 1 },
                        }}
                    >
                        <Card className="group h-full border-2 transition-all duration-300 hover:scale-[1.02] hover:border-primary/50 hover:shadow-xl">
                            <CardContent className="p-6">
                                <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10 transition-colors group-hover:bg-primary/20">
                                    <Shield className="h-6 w-6 text-primary" />
                                </div>
                                <h3 className="mb-2 font-semibold text-xl">Team Builder</h3>
                                <p className="text-muted-foreground">Create and optimize squad compositions for any stage or challenge.</p>
                            </CardContent>
                        </Card>
                    </InView>

                    <InView
                        transition={{ duration: 0.5, delay: 0.6 }}
                        variants={{
                            hidden: { opacity: 0, y: 30, scale: 0.95 },
                            visible: { opacity: 1, y: 0, scale: 1 },
                        }}
                    >
                        <Card className="group h-full border-2 transition-all duration-300 hover:scale-[1.02] hover:border-primary/50 hover:shadow-xl">
                            <CardContent className="p-6">
                                <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10 transition-colors group-hover:bg-primary/20">
                                    <Users className="h-6 w-6 text-primary" />
                                </div>
                                <h3 className="mb-2 font-semibold text-xl">Community Hub</h3>
                                <p className="text-muted-foreground">Connect with other Doctors, share strategies, and join discussions.</p>
                            </CardContent>
                        </Card>
                    </InView>
                </div>
            </section>

            {/* CTA Section */}
            <section className="px-4 py-20">
                <InView
                    transition={{ duration: 0.6 }}
                    variants={{
                        hidden: { opacity: 0, scale: 0.95 },
                        visible: { opacity: 1, scale: 1 },
                    }}
                >
                    <Card className="mx-auto max-w-4xl border-2 border-primary/30 bg-gradient-to-br from-primary/5 to-background">
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
        </div>
    );
}
