"use client";

import { motion } from "motion/react";

interface TagsDisplayProps {
    tags: string[];
}

export function TagsDisplay({ tags }: TagsDisplayProps) {
    if (!tags || tags.length === 0) return null;

    return (
        <div className="space-y-2">
            <h3 className="text-lg font-semibold">Tags</h3>
            <div className="flex flex-wrap gap-2">
                {tags.map((tag, index) => (
                    <motion.span key={tag} initial={{ opacity: 0, scale: 0.9 }} animate={{ opacity: 1, scale: 1 }} transition={{ delay: index * 0.05 }} className="rounded-md bg-muted px-3 py-1.5 text-sm font-medium">
                        {tag}
                    </motion.span>
                ))}
            </div>
        </div>
    );
}
