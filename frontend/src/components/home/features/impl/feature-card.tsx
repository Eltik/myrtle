import Link from "next/link";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import type { Feature } from "./constants";
import { ANIMATION_VARIANTS, getCardDelay } from "./constants";

interface FeatureCardProps {
    feature: Feature;
    index: number;
}

export function FeatureCard({ feature, index }: FeatureCardProps) {
    const Icon = feature.icon;
    const delay = getCardDelay(index);

    const cardContent = (
        <Card className="group h-full cursor-pointer border-2 transition-all duration-300 hover:scale-[1.02] hover:border-primary/50 hover:shadow-xl">
            <CardContent className="p-6">
                <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10 transition-colors group-hover:bg-primary/20">
                    <Icon className="h-6 w-6 text-primary" />
                </div>
                <h3 className="mb-2 font-semibold text-xl">{feature.title}</h3>
                <p className="text-muted-foreground">{feature.description}</p>
            </CardContent>
        </Card>
    );

    return (
        <InView once transition={{ duration: 0.5, delay }} variants={ANIMATION_VARIANTS.card}>
            {feature.href ? <Link href={feature.href}>{cardContent}</Link> : cardContent}
        </InView>
    );
}
