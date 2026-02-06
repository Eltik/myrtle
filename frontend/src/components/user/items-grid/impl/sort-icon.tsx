"use client";

import { ArrowDown, ArrowUp, ArrowUpDown } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";

interface SortIconProps {
    field: "name" | "amount" | "rarity" | "category";
    sortBy: "name" | "amount" | "rarity" | "category";
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
                    <motion.div animate={{ opacity: 1, scaleY: 1 }} className="absolute inset-0" exit={{ opacity: 0, scaleY: -1 }} initial={{ opacity: 0, scaleY: -1 }} key={`${field}-${isAscending ? "asc" : "desc"}`} transition={{ duration: 0.15, ease: "easeOut" }}>
                        {isAscending ? <ArrowUp className="h-3.5 w-3.5" /> : <ArrowDown className="h-3.5 w-3.5" />}
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    );
}
