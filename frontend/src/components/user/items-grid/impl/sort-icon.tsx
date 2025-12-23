"use client";

import { ArrowDown, ArrowUp, ArrowUpDown } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";

interface SortIconProps {
    field: "name" | "amount";
    sortBy: "name" | "amount";
    sortOrder: "asc" | "desc";
}

export function SortIcon({ field, sortBy, sortOrder }: SortIconProps) {
    const isActive = sortBy === field;
    const isAscending = sortOrder === "asc";

    return (
        <div className="relative h-3.5 w-3.5">
            <AnimatePresence mode="wait">
                {!isActive ? (
                    <motion.div animate={{ opacity: 0.5, scale: 1 }} className="absolute inset-0" exit={{ opacity: 0, scale: 0.8 }} initial={{ opacity: 0, scale: 0.8 }} key="inactive" transition={{ duration: 0.15 }}>
                        <ArrowUpDown className="h-3.5 w-3.5" />
                    </motion.div>
                ) : (
                    <motion.div animate={{ opacity: 1, rotate: 0 }} className="absolute inset-0" exit={{ opacity: 0, rotate: isAscending ? -180 : 180 }} initial={{ opacity: 0, rotate: isAscending ? 180 : -180 }} key={isAscending ? "asc" : "desc"} transition={{ duration: 0.2, ease: [0.25, 0.46, 0.45, 0.94] }}>
                        {isAscending ? <ArrowUp className="h-3.5 w-3.5" /> : <ArrowDown className="h-3.5 w-3.5" />}
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    );
}
