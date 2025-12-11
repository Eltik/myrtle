"use client";

import { Eye } from "lucide-react";
import { motion } from "motion/react";

export function ChibiViewerPlaceholder() {
    return (
        <motion.div className="relative aspect-square w-48 overflow-hidden rounded-xl border border-muted-foreground/30 border-dashed bg-card/50 backdrop-blur-sm" whileHover={{ scale: 1.02 }}>
            <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 p-4">
                <motion.div
                    animate={{
                        y: [0, -4, 0],
                        opacity: [0.5, 1, 0.5],
                    }}
                    className="rounded-full bg-primary/20 p-4"
                    transition={{
                        duration: 2,
                        repeat: Number.POSITIVE_INFINITY,
                        ease: "easeInOut",
                    }}
                >
                    <Eye className="h-8 w-8 text-primary" />
                </motion.div>
                <div className="text-center">
                    <p className="font-medium text-foreground text-sm">Chibi Viewer</p>
                    <p className="text-muted-foreground text-xs">Coming Soon</p>
                </div>
            </div>

            {/* Animated border glow */}
            <div className="pointer-events-none absolute inset-0 rounded-xl opacity-50">
                <motion.div
                    animate={{
                        background: ["linear-gradient(0deg, hsl(var(--primary) / 0.2) 0%, transparent 50%)", "linear-gradient(180deg, hsl(var(--primary) / 0.2) 0%, transparent 50%)", "linear-gradient(360deg, hsl(var(--primary) / 0.2) 0%, transparent 50%)"],
                    }}
                    className="absolute inset-0 rounded-xl"
                    transition={{ duration: 3, repeat: Number.POSITIVE_INFINITY }}
                />
            </div>
        </motion.div>
    );
}
