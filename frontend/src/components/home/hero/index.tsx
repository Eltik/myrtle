"use client";

import { ArrowRight, BookOpen } from "lucide-react";
import Image from "next/image";
import Link from "next/link";
import { Marquee } from "~/components/ui/marquee";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { TextLoop } from "~/components/ui/motion-primitives/text-loop";
import { Button } from "~/components/ui/shadcn/button";
import { ANIMATION_TRANSITIONS, ANIMATION_VARIANTS, HERO_IMAGES_PRIMARY, HERO_IMAGES_SECONDARY, HERO_KEYWORDS } from "./impl/constants";

export function HeroSection() {
    return (
        <section className="hero-full-width -mt-8 relative flex min-h-[80vh] flex-col items-center justify-center overflow-hidden pt-8 pb-20 md:mt-0 md:py-20">
            {/* Parallax Background Images */}
            <div className="absolute inset-0 z-1 flex flex-col justify-center">
                <Marquee className="mb-4" duration={120}>
                    {HERO_IMAGES_PRIMARY.map((src, index) => (
                        <Image alt="" className="hero-image rounded-lg object-cover" height={300} key={src} loading="eager" priority={index < 3} src={src} width={400} />
                    ))}
                </Marquee>
                <Marquee className="mt-4" duration={140} reverse>
                    {HERO_IMAGES_SECONDARY.map((src) => (
                        <Image alt="" className="hero-image-secondary rounded-lg object-cover" height={300} key={src} loading="eager" src={src} width={400} />
                    ))}
                </Marquee>
            </div>

            {/* Hero Content */}
            <div className="relative z-10 mx-auto max-w-4xl px-4 text-center">
                <InView transition={ANIMATION_TRANSITIONS.headline} variants={ANIMATION_VARIANTS.headline}>
                    <h1 className="mb-6 font-bold text-5xl leading-tight md:text-7xl">
                        <span className="block">Sync, discover, and optimize your</span>
                        <span className="relative inline-block">
                            <span aria-hidden="true" className="invisible select-none">
                                recruitment
                            </span>
                            <TextLoop
                                className="absolute inset-0 text-primary"
                                interval={2.5}
                                style={{
                                    textShadow: "0 0 20px var(--glow-text-strong), 0 0 40px var(--glow-text-medium), 0 0 60px var(--glow-text-soft)",
                                }}
                            >
                                {HERO_KEYWORDS.map((keyword) => (
                                    <span key={keyword}>{keyword}</span>
                                ))}
                            </TextLoop>
                        </span>
                    </h1>
                </InView>

                <InView transition={ANIMATION_TRANSITIONS.subtitle} variants={ANIMATION_VARIANTS.subtitle}>
                    <p className="mx-auto mb-8 max-w-2xl text-balance text-muted-foreground text-xl md:text-2xl">The ultimate toolkit for Arknights Doctors.</p>
                </InView>

                <InView transition={ANIMATION_TRANSITIONS.buttons} variants={ANIMATION_VARIANTS.buttons}>
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
                                Get Started
                            </Link>
                        </Button>
                    </div>
                </InView>
            </div>
        </section>
    );
}
