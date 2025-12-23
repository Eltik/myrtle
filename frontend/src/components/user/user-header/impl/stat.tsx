"use client";

import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";

interface StatProps {
    label: string;
    value: number;
}

export function Stat({ label, value }: StatProps) {
    return (
        <div className="text-center">
            <div className="font-bold text-2xl">
                <AnimatedNumber springOptions={{ bounce: 0, duration: 1000 }} value={value} />
            </div>
            <div className="text-muted-foreground text-sm">{label}</div>
        </div>
    );
}
