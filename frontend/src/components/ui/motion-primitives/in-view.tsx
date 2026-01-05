"use client";
import { motion, type Transition, type UseInViewOptions, useInView, type Variant } from "motion/react";
import { type ReactNode, useMemo, useRef } from "react";

export type InViewProps = {
    children: ReactNode;
    variants?: {
        hidden: Variant;
        visible: Variant;
    };
    transition?: Transition;
    viewOptions?: UseInViewOptions;
    as?: React.ElementType;
    once?: boolean;
};

const defaultVariants = {
    hidden: { opacity: 0 },
    visible: { opacity: 1 },
};

export function InView({ children, variants = defaultVariants, transition, viewOptions, as = "div", once }: InViewProps) {
    const ref = useRef(null);

    const mergedViewOptions = useMemo(
        () => ({
            ...viewOptions,
            once: once ?? viewOptions?.once,
        }),
        [viewOptions, once],
    );
    const isInView = useInView(ref, mergedViewOptions);

    const MotionComponent = motion[as as keyof typeof motion] as typeof as;

    return (
        <MotionComponent animate={isInView ? "visible" : "hidden"} initial="hidden" ref={ref} transition={transition} variants={variants}>
            {children}
        </MotionComponent>
    );
}
