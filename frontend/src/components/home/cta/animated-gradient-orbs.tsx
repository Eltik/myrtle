"use client";

import { motion, type SpringOptions, useMotionValue, useSpring, useTransform } from "motion/react";
import { useCallback, useEffect, useRef, useState } from "react";
import { cn } from "~/lib/utils";

type AnimatedGradientOrbsProps = {
    className?: string;
    cursorInfluence?: number;
    springOptions?: SpringOptions;
};

const DEFAULT_SPRING_OPTIONS: SpringOptions = {
    stiffness: 50,
    damping: 20,
    mass: 1,
};

// Orb constants
const ORB1_BASE_X = 85;
const ORB1_BASE_Y = -5;
const ORB1_SIZE = 280;
const ORB1_AUTO_RADIUS = 35;
const ORB1_AUTO_SPEED = 0.0008;

const ORB2_BASE_X = -5;
const ORB2_BASE_Y = 85;
const ORB2_SIZE = 280;
const ORB2_AUTO_RADIUS = 30;
const ORB2_AUTO_SPEED = 0.0006;

// Speed multiplier when cursor is influencing (0 = stopped, 1 = full speed)
const HOVER_SPEED_MULTIPLIER = 0.25;

export function AnimatedGradientOrbs({ className, cursorInfluence = 0.15, springOptions = DEFAULT_SPRING_OPTIONS }: AnimatedGradientOrbsProps) {
    const containerRef = useRef<HTMLDivElement>(null);
    const [parentElement, setParentElement] = useState<HTMLElement | null>(null);
    const [dimensions, setDimensions] = useState({ width: 0, height: 0 });
    const animationFrameRef = useRef<number>(0);

    // Track hover state and virtual time for variable-speed animation
    const isHoveredRef = useRef(false);
    const virtualTimeRef = useRef(0);
    const lastTimestampRef = useRef(0);
    const currentSpeedRef = useRef(1);

    // Mouse position relative to center (normalized -1 to 1)
    const mouseXRaw = useMotionValue(0);
    const mouseYRaw = useMotionValue(0);

    // Smoothed mouse position with spring
    const mouseX = useSpring(mouseXRaw, springOptions);
    const mouseY = useSpring(mouseYRaw, springOptions);

    // Auto-animation offset values
    const autoOffsetX1 = useMotionValue(0);
    const autoOffsetY1 = useMotionValue(0);
    const autoOffsetX2 = useMotionValue(0);
    const autoOffsetY2 = useMotionValue(0);

    // Setup parent element reference and dimensions
    useEffect(() => {
        if (containerRef.current) {
            const parent = containerRef.current.parentElement;
            if (parent) {
                setParentElement(parent);
                const rect = parent.getBoundingClientRect();
                setDimensions({ width: rect.width, height: rect.height });

                const resizeObserver = new ResizeObserver((entries) => {
                    for (const entry of entries) {
                        setDimensions({
                            width: entry.contentRect.width,
                            height: entry.contentRect.height,
                        });
                    }
                });

                resizeObserver.observe(parent);
                return () => resizeObserver.disconnect();
            }
        }
    }, []);

    // Mouse movement handler
    const handleMouseMove = useCallback(
        (event: MouseEvent) => {
            if (!parentElement) return;
            const rect = parentElement.getBoundingClientRect();
            const centerX = rect.left + rect.width / 2;
            const centerY = rect.top + rect.height / 2;

            // Normalize to -1 to 1 range
            const normalizedX = (event.clientX - centerX) / (rect.width / 2);
            const normalizedY = (event.clientY - centerY) / (rect.height / 2);

            mouseXRaw.set(normalizedX);
            mouseYRaw.set(normalizedY);
        },
        [parentElement, mouseXRaw, mouseYRaw],
    );

    const handleMouseEnter = useCallback(() => {
        isHoveredRef.current = true;
    }, []);

    const handleMouseLeave = useCallback(() => {
        isHoveredRef.current = false;
        // Smoothly return to center
        mouseXRaw.set(0);
        mouseYRaw.set(0);
    }, [mouseXRaw, mouseYRaw]);

    // Mouse event listeners
    useEffect(() => {
        if (!parentElement) return;

        const abortController = new AbortController();

        parentElement.addEventListener("mousemove", handleMouseMove, {
            signal: abortController.signal,
        });
        parentElement.addEventListener("mouseenter", handleMouseEnter, {
            signal: abortController.signal,
        });
        parentElement.addEventListener("mouseleave", handleMouseLeave, {
            signal: abortController.signal,
        });

        return () => abortController.abort();
    }, [parentElement, handleMouseMove, handleMouseEnter, handleMouseLeave]);

    // Auto-animation loop with variable speed
    useEffect(() => {
        const animate = (timestamp: number) => {
            // Calculate delta time
            if (lastTimestampRef.current === 0) {
                lastTimestampRef.current = timestamp;
            }
            const deltaTime = timestamp - lastTimestampRef.current;
            lastTimestampRef.current = timestamp;

            // Smoothly interpolate speed multiplier
            const targetSpeed = isHoveredRef.current ? HOVER_SPEED_MULTIPLIER : 1;
            currentSpeedRef.current += (targetSpeed - currentSpeedRef.current) * 0.05;

            // Accumulate virtual time at variable rate
            virtualTimeRef.current += deltaTime * currentSpeedRef.current;
            const vTime = virtualTimeRef.current;

            // Orb 1 - figure-8 pattern
            const t1 = vTime * ORB1_AUTO_SPEED;
            autoOffsetX1.set(Math.sin(t1) * ORB1_AUTO_RADIUS);
            autoOffsetY1.set(Math.sin(t1 * 2) * ORB1_AUTO_RADIUS * 0.5);

            // Orb 2 - circular pattern with slight variation (offset by PI)
            const t2 = vTime * ORB2_AUTO_SPEED + Math.PI;
            autoOffsetX2.set(Math.cos(t2) * ORB2_AUTO_RADIUS);
            autoOffsetY2.set(Math.sin(t2 * 1.5) * ORB2_AUTO_RADIUS * 0.7);

            animationFrameRef.current = requestAnimationFrame(animate);
        };

        animationFrameRef.current = requestAnimationFrame(animate);
        return () => cancelAnimationFrame(animationFrameRef.current);
    }, [autoOffsetX1, autoOffsetY1, autoOffsetX2, autoOffsetY2]);

    // Calculate cursor influence on each orb
    const cursorInfluenceX = dimensions.width * cursorInfluence;
    const cursorInfluenceY = dimensions.height * cursorInfluence;

    // Transform functions for final positions
    const orb1Left = useTransform([mouseX, autoOffsetX1], (values: number[]) => {
        const mouse = values[0] ?? 0;
        const auto = values[1] ?? 0;
        return `calc(${ORB1_BASE_X}% + ${auto + mouse * cursorInfluenceX}px)`;
    });

    const orb1Top = useTransform([mouseY, autoOffsetY1], (values: number[]) => {
        const mouse = values[0] ?? 0;
        const auto = values[1] ?? 0;
        return `calc(${ORB1_BASE_Y}% + ${auto + mouse * cursorInfluenceY}px)`;
    });

    const orb2Left = useTransform([mouseX, autoOffsetX2], (values: number[]) => {
        const mouse = values[0] ?? 0;
        const auto = values[1] ?? 0;
        return `calc(${ORB2_BASE_X}% + ${auto + mouse * cursorInfluenceX * 0.7}px)`;
    });

    const orb2Top = useTransform([mouseY, autoOffsetY2], (values: number[]) => {
        const mouse = values[0] ?? 0;
        const auto = values[1] ?? 0;
        return `calc(${ORB2_BASE_Y}% + ${auto + mouse * cursorInfluenceY * 0.7}px)`;
    });

    return (
        <div className={cn("pointer-events-none absolute inset-0 overflow-hidden", className)} ref={containerRef}>
            {/* Orb 1 - Top Right with metallic effect */}
            <motion.div
                className="absolute rounded-full blur-3xl"
                style={{
                    left: orb1Left,
                    top: orb1Top,
                    width: ORB1_SIZE,
                    height: ORB1_SIZE,
                    willChange: "transform",
                    background: `
                        radial-gradient(ellipse 60% 40% at 30% 30%, oklch(0.85 0.18 25 / 0.4) 0%, transparent 50%),
                        radial-gradient(ellipse 80% 80% at 50% 50%, oklch(0.65 0.20 25 / 0.25) 0%, oklch(0.55 0.22 25 / 0.15) 50%, transparent 70%),
                        radial-gradient(circle at 70% 70%, oklch(0.45 0.18 25 / 0.2) 0%, transparent 50%)
                    `,
                }}
            />
            {/* Orb 1 highlight layer */}
            <motion.div
                className="absolute rounded-full blur-2xl"
                style={{
                    left: orb1Left,
                    top: orb1Top,
                    width: ORB1_SIZE * 0.6,
                    height: ORB1_SIZE * 0.5,
                    willChange: "transform",
                    background: "radial-gradient(ellipse at 40% 40%, oklch(0.95 0.10 25 / 0.3) 0%, transparent 60%)",
                    transform: "translate(15%, 10%)",
                }}
            />

            {/* Orb 2 - Bottom Left with metallic effect */}
            <motion.div
                className="absolute rounded-full blur-3xl"
                style={{
                    left: orb2Left,
                    top: orb2Top,
                    width: ORB2_SIZE,
                    height: ORB2_SIZE,
                    willChange: "transform",
                    background: `
                        radial-gradient(ellipse 60% 40% at 70% 30%, oklch(0.80 0.15 25 / 0.35) 0%, transparent 50%),
                        radial-gradient(ellipse 80% 80% at 50% 50%, oklch(0.60 0.18 25 / 0.2) 0%, oklch(0.50 0.20 25 / 0.12) 50%, transparent 70%),
                        radial-gradient(circle at 30% 70%, oklch(0.40 0.15 25 / 0.18) 0%, transparent 50%)
                    `,
                }}
            />
            {/* Orb 2 highlight layer */}
            <motion.div
                className="absolute rounded-full blur-2xl"
                style={{
                    left: orb2Left,
                    top: orb2Top,
                    width: ORB2_SIZE * 0.5,
                    height: ORB2_SIZE * 0.4,
                    willChange: "transform",
                    background: "radial-gradient(ellipse at 60% 35%, oklch(0.92 0.08 25 / 0.25) 0%, transparent 55%)",
                    transform: "translate(25%, 15%)",
                }}
            />
        </div>
    );
}
