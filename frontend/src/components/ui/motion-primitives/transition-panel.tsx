"use client";
import { AnimatePresence, type MotionProps, motion, type Transition, type Variant } from "motion/react";
import useMeasure from "react-use-measure";
import { cn } from "~/lib/utils";

export type TransitionPanelProps = {
    children: React.ReactNode[];
    className?: string;
    transition?: Transition;
    activeIndex: number;
    variants?: { enter: Variant; center: Variant; exit: Variant };
    animateHeight?: boolean;
    heightTransition?: Transition;
} & MotionProps;

export function TransitionPanel({ children, className, transition, variants, activeIndex, animateHeight = false, heightTransition, ...motionProps }: TransitionPanelProps) {
    const [ref, bounds] = useMeasure();

    return (
        <motion.div animate={{ height: animateHeight && bounds.height > 0 ? bounds.height : "auto" }} className={cn("relative overflow-hidden", className)} transition={heightTransition ?? { type: "spring", bounce: 0.15, duration: 0.5 }}>
            <AnimatePresence custom={motionProps.custom} initial={false} mode="popLayout">
                <motion.div animate="center" exit="exit" initial="enter" key={activeIndex} ref={ref} transition={transition} variants={variants} {...motionProps}>
                    {children[activeIndex]}
                </motion.div>
            </AnimatePresence>
        </motion.div>
    );
}
