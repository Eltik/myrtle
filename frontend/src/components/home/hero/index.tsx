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
        <section className="hero-full-width hero-light-bg relative -mt-8 flex min-h-[80vh] flex-col items-center justify-center overflow-hidden pt-8 pb-20 md:mt-0 md:py-20">
            {/* Parallax Background Images */}
            <div className="hero-images-container absolute inset-0 z-1 flex flex-col justify-center">
                <Marquee className="mb-4" duration={120}>
                    {HERO_IMAGES_PRIMARY.map((src, index) => (
                        <Image alt="Arknights artwork" className="hero-image rounded-lg object-cover" height={300} key={src} loading={index < 2 ? "eager" : "lazy"} priority={index < 2} quality={75} sizes="400px" src={src} width={400} />
                    ))}
                </Marquee>
                <Marquee className="mt-4" duration={140} reverse>
                    {HERO_IMAGES_SECONDARY.map((src) => (
                        <Image alt="Arknights artwork" className="hero-image-secondary rounded-lg object-cover" height={300} key={src} loading="lazy" priority={false} quality={75} sizes="400px" src={src} width={400} />
                    ))}
                </Marquee>
            </div>

            {/* Radial vignette - spotlight effect on content, images visible at edges */}
            <div className="hero-vignette absolute inset-0 z-2" />
            <div className="relative z-10 mx-auto max-w-4xl px-4 text-center">
                <InView once transition={ANIMATION_TRANSITIONS.headline} variants={ANIMATION_VARIANTS.headline}>
                    <h1 className="mb-6 font-bold text-5xl leading-tight md:text-7xl">
                        <span className="block">Sync, discover, and optimize your</span>
                        <span className="relative inline-block">
                            <span aria-hidden="true" className="invisible select-none">
                                recruitment
                            </span>
                            <TextLoop className="hero-accent-glow absolute inset-0 text-primary" interval={3.5}>
                                {HERO_KEYWORDS.map((keyword) => (
                                    <span key={keyword}>{keyword}</span>
                                ))}
                            </TextLoop>
                        </span>
                    </h1>
                </InView>

                <InView once transition={ANIMATION_TRANSITIONS.subtitle} variants={ANIMATION_VARIANTS.subtitle}>
                    <p className="mx-auto mb-8 max-w-2xl text-balance text-muted-foreground text-xl md:text-2xl">The ultimate toolkit for Arknights Doctors.</p>
                </InView>

                <InView once transition={ANIMATION_TRANSITIONS.buttons} variants={ANIMATION_VARIANTS.buttons}>
                    <div className="flex flex-col items-center justify-center gap-4 sm:flex-row">
                        <Button asChild className="group relative h-12 overflow-hidden rounded-lg bg-primary px-8 font-semibold text-lg text-primary-foreground shadow-[0_8px_24px_var(--glow-primary)] transition-all duration-300 hover:scale-[1.02] hover:shadow-[0_12px_32px_var(--glow-primary)]" size="lg">
                            <Link href="/operators">
                                <span className="pointer-events-none absolute inset-x-0 top-0 h-1/2 bg-linear-to-b from-white/25 to-transparent" />
                                <span className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-linear-to-t from-black/20 to-transparent" />
                                <span className="relative flex items-center">
                                    Browse Operators
                                    <ArrowRight className="ml-2 h-5 w-5 transition-transform group-hover:translate-x-1" />
                                </span>
                            </Link>
                        </Button>
                        <Button
                            className="h-12 cursor-pointer rounded-lg border-2 border-primary/30 bg-secondary/50 px-8 font-semibold text-foreground text-lg transition-all duration-300 hover:border-primary/60 hover:bg-secondary"
                            onClick={() => document.getElementById("features")?.scrollIntoView({ behavior: "smooth" })}
                            size="lg"
                            variant="outline"
                        >
                            <BookOpen className="mr-2 h-5 w-5" />
                            Get Started
                        </Button>
                    </div>
                </InView>
            </div>
        </section>
    );
}
