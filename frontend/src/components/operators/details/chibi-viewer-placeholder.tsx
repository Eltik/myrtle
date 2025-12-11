"use client";

import { motion } from "motion/react";
import { Eye } from "lucide-react";

export function ChibiViewerPlaceholder() {
    return (
        <motion.div whileHover={{ scale: 1.02 }} className="relative aspect-square w-48 overflow-hidden rounded-xl border border-dashed border-muted-foreground/30 bg-card/50 backdrop-blur-sm">
            <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 p-4">
                <motion.div
                    animate={{
                        y: [0, -4, 0],
                        opacity: [0.5, 1, 0.5],
                    }}
                    transition={{
                        duration: 2,
                        repeat: Number.POSITIVE_INFINITY,
                        ease: "easeInOut",
                    }}
                    className="rounded-full bg-primary/20 p-4"
                >
                    <Eye className="h-8 w-8 text-primary" />
                </motion.div>
                <div className="text-center">
                    <p className="text-sm font-medium text-foreground">Chibi Viewer</p>
                    <p className="text-xs text-muted-foreground">Coming Soon</p>
                </div>
            </div>

            {/* Animated border glow */}
            <div className="pointer-events-none absolute inset-0 rounded-xl opacity-50">
                <motion.div
                    animate={{
                        background: ["linear-gradient(0deg, hsl(var(--primary) / 0.2) 0%, transparent 50%)", "linear-gradient(180deg, hsl(var(--primary) / 0.2) 0%, transparent 50%)", "linear-gradient(360deg, hsl(var(--primary) / 0.2) 0%, transparent 50%)"],
                    }}
                    transition={{ duration: 3, repeat: Number.POSITIVE_INFINITY }}
                    className="absolute inset-0 rounded-xl"
                />
            </div>
        </motion.div>
    );
}
