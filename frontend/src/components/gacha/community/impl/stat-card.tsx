import type { LucideIcon } from "lucide-react";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";

interface StatCardProps {
    title: string;
    value: number;
    description: string;
    icon: LucideIcon;
    iconClassName?: string;
    valueClassName?: string;
    delay?: number;
}

export function StatCard({ title, value, description, icon: Icon, iconClassName = "text-muted-foreground", valueClassName, delay = 0.1 }: StatCardProps) {
    return (
        <InView
            once
            transition={{ duration: 0.5, ease: "easeOut", delay }}
            variants={{
                hidden: { opacity: 0, scale: 0.9 },
                visible: { opacity: 1, scale: 1 },
            }}
        >
            <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle className="font-medium text-sm">{title}</CardTitle>
                    <Icon className={`h-4 w-4 ${iconClassName}`} />
                </CardHeader>
                <CardContent>
                    <div className={`font-bold text-2xl ${valueClassName ?? ""}`}>
                        <AnimatedNumber springOptions={{ bounce: 0, duration: 2000 }} value={value} />
                    </div>
                    <p className="text-muted-foreground text-xs">{description}</p>
                </CardContent>
            </Card>
        </InView>
    );
}
