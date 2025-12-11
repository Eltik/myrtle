"use client";

import { motion } from "motion/react";

interface OperatorTagsProps {
    tags: string[];
}

export function OperatorTags({ tags }: OperatorTagsProps) {
    if (!tags || tags.length === 0) return null;

    return (
        <div className="rounded-xl border bg-card/50 p-6 backdrop-blur-sm">
            <h3 className="mb-4 text-lg font-semibold">Tags</h3>
            <div className="flex flex-wrap gap-2">
                {tags.map((tag, index) => (
                    <motion.span key={tag} initial={{ opacity: 0, scale: 0.8 }} animate={{ opacity: 1, scale: 1 }} transition={{ delay: index * 0.05 }} className="rounded-full bg-muted px-3 py-1.5 text-sm">
                        {tag}
                    </motion.span>
                ))}
            </div>
        </div>
    );
}
