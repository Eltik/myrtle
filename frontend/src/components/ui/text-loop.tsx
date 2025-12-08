"use client";
import { AnimatePresence, type AnimatePresenceProps, motion, type Transition, type Variants } from "motion/react";
import { Children, useEffect, useState } from "react";
import { cn } from "~/lib/utils";

export type TextLoopProps = {
    children: React.ReactNode[];
    className?: string;
    style?: React.CSSProperties;
    interval?: number;
    transition?: Transition;
    variants?: Variants;
    onIndexChange?: (index: number) => void;
    trigger?: boolean;
    mode?: AnimatePresenceProps["mode"];
};

export function TextLoop({ children, className, style, interval = 2, transition = { duration: 0.3 }, variants, onIndexChange, trigger = true, mode = "popLayout" }: TextLoopProps) {
    const [currentIndex, setCurrentIndex] = useState(0);
    const items = Children.toArray(children);

    useEffect(() => {
        if (!trigger) return;

        const intervalMs = interval * 1000;
        const timer = setInterval(() => {
            setCurrentIndex((current) => {
                const next = (current + 1) % items.length;
                onIndexChange?.(next);
                return next;
            });
        }, intervalMs);
        return () => clearInterval(timer);
    }, [items.length, interval, onIndexChange, trigger]);

    const motionVariants: Variants = {
        initial: { y: 20, opacity: 0, filter: "blur(8px)" },
        animate: { y: 0, opacity: 1, filter: "blur(0px)" },
        exit: { y: -20, opacity: 0, filter: "blur(8px)" },
    };

    return (
        <div className={cn("relative inline-block whitespace-nowrap", className)} style={style}>
            <AnimatePresence initial={false} mode={mode}>
                <motion.div animate="animate" exit="exit" initial="initial" key={currentIndex} transition={transition} variants={variants || motionVariants}>
                    {items[currentIndex]}
                </motion.div>
            </AnimatePresence>
        </div>
    );
}
