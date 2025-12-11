"use client";

import { motion } from "motion/react";
import Image from "next/image";
import type { Operator } from "~/types/api";
import { getOperatorImage } from "~/lib/operator-helpers";
import { TextEffect } from "~/components/ui/motion-primitives/text-effect";
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbSeparator } from "~/components/ui/shadcn/breadcrumb";
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
            <motion.div initial={{ scale: 1.1, opacity: 0 }} animate={{ scale: 1, opacity: 1 }} transition={{ duration: 0.8, ease: "easeOut" }} className="absolute inset-0">
                <Image src={imageUrl || "/placeholder.svg"} alt={operator.name} fill className="object-cover object-top" priority />
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
                <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.3, duration: 0.5 }}>
                    <Breadcrumb className="mb-4">
                        <BreadcrumbList>
                            <BreadcrumbItem>
                                <BreadcrumbLink href="/operators" className="text-muted-foreground hover:text-foreground">
                                    Operators
                                </BreadcrumbLink>
                            </BreadcrumbItem>
                            <BreadcrumbSeparator />
                            <BreadcrumbItem>
                                <BreadcrumbLink href={`/operators?id=${operator.id}`} className="text-foreground">
                                    {operator.name}
                                </BreadcrumbLink>
                            </BreadcrumbItem>
                        </BreadcrumbList>
                    </Breadcrumb>

                    <TextEffect preset="fade-in-blur" per="char" as="h1" className="text-4xl font-bold tracking-tight text-foreground md:text-6xl lg:text-7xl" speedReveal={1.5}>
                        {operator.name}
                    </TextEffect>
                </motion.div>

                {/* Chibi Viewer Placeholder */}
                <motion.div initial={{ opacity: 0, scale: 0.9 }} animate={{ opacity: 1, scale: 1 }} transition={{ delay: 0.5, duration: 0.5 }} className="absolute bottom-32 right-4 hidden md:right-8 lg:block">
                    <ChibiViewerPlaceholder />
                </motion.div>
            </div>
        </div>
    );
}
