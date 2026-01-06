"use client";
import { motion, type SpringOptions, useSpring, useTransform } from "motion/react";
import { useEffect } from "react";
import { cn } from "~/lib/utils";

export type AnimatedNumberProps = {
    value: number;
    className?: string;
    springOptions?: SpringOptions;
    as?: React.ElementType;
    decimals?: number;
};

export function AnimatedNumber({ value, className, springOptions, as: Component = "span", decimals = 0 }: AnimatedNumberProps) {
    const MotionComponent = motion(Component);

    const spring = useSpring(0, springOptions);
    const display = useTransform(spring, (current) => {
        if (decimals > 0) {
            return current.toFixed(decimals);
        }
        return Math.round(current).toLocaleString();
    });

    useEffect(() => {
        spring.set(value);
    }, [spring, value]);

    return <MotionComponent className={cn("tabular-nums", className)}>{display}</MotionComponent>;
}
