"use client";

import { motion } from "motion/react";
import Image from "next/image";
import { TextEffect } from "~/components/ui/motion-primitives/text-effect";
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbSeparator } from "~/components/ui/shadcn/breadcrumb";
import { getOperatorImage } from "~/lib/operator-helpers";
import type { Operator } from "~/types/api";
import { ChibiViewerPlaceholder } from "./chibi-viewer-placeholder";

interface OperatorHeroProps {
    operator: Operator;
}

export function OperatorHero({ operator }: OperatorHeroProps) {
    const phaseCount = operator.phases?.length ?? 1;
    const imageUrl = getOperatorImage(operator.id ?? "", phaseCount > 2 ? 2 : 1);

    return (
        <div className="relative h-[600px] w-full overflow-hidden md:h-[700px]">
            {/* Background Image */}
            <motion.div animate={{ scale: 1, opacity: 1 }} className="absolute inset-0" initial={{ scale: 1.1, opacity: 0 }} transition={{ duration: 0.8, ease: "easeOut" }}>
                <Image alt={operator.name} className="object-cover object-top" fill priority src={imageUrl || "/placeholder.svg"} />
            </motion.div>

            {/* Gradient Overlays */}
            <div
                className="absolute inset-0 z-10"
                style={{
                    background: "linear-gradient(180deg, rgba(0,0,0,0) 0%, rgba(0,0,0,0.3) 50%, hsl(var(--background)) 100%)",
                }}
            />
            <div
                className="absolute inset-0 z-10"
                style={{
                    background: "linear-gradient(90deg, hsl(var(--background) / 0.5) 0%, transparent 50%)",
                }}
            />

            {/* Content */}
            <div className="container relative z-20 mx-auto flex h-full flex-col justify-end px-4 pb-40">
                <motion.div animate={{ opacity: 1, y: 0 }} initial={{ opacity: 0, y: 20 }} transition={{ delay: 0.3, duration: 0.5 }}>
                    <Breadcrumb className="mb-4">
                        <BreadcrumbList>
                            <BreadcrumbItem>
                                <BreadcrumbLink className="text-muted-foreground hover:text-foreground" href="/operators">
                                    Operators
                                </BreadcrumbLink>
                            </BreadcrumbItem>
                            <BreadcrumbSeparator />
                            <BreadcrumbItem>
                                <BreadcrumbLink className="text-foreground" href={`/operators?id=${operator.id}`}>
                                    {operator.name}
                                </BreadcrumbLink>
                            </BreadcrumbItem>
                        </BreadcrumbList>
                    </Breadcrumb>

                    <TextEffect as="h1" className="font-bold text-4xl text-foreground tracking-tight md:text-6xl lg:text-7xl" per="char" preset="fade-in-blur" speedReveal={1.5}>
                        {operator.name}
                    </TextEffect>
                </motion.div>

                {/* Chibi Viewer Placeholder */}
                <motion.div animate={{ opacity: 1, scale: 1 }} className="absolute right-4 bottom-32 hidden md:right-8 lg:block" initial={{ opacity: 0, scale: 0.9 }} transition={{ delay: 0.5, duration: 0.5 }}>
                    <ChibiViewerPlaceholder />
                </motion.div>
            </div>
        </div>
    );
}
