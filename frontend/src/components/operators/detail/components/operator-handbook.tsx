"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "motion/react";
import type { Operator } from "~/types/api";
import { cn } from "~/lib/utils";

interface OperatorHandbookProps {
    operator: Operator;
}

export function OperatorHandbook({ operator }: OperatorHandbookProps) {
    const [activeSection, setActiveSection] = useState(0);

    const handbook = operator.handbook;
    if (!handbook?.storyTextAudio) {
        return <p className="py-4 text-sm text-muted-foreground">No handbook data available.</p>;
    }

    const stories = handbook.storyTextAudio;

    return (
        <div className="space-y-4 pt-4">
            {/* Section tabs */}
            <div className="flex flex-wrap gap-2">
                {stories.map((story, index) => (
                    <button key={index} onClick={() => setActiveSection(index)} className={cn("rounded-lg px-3 py-1.5 text-sm transition-colors", activeSection === index ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground hover:text-foreground")}>
                        {story.storyTitle ?? `File ${index + 1}`}
                    </button>
                ))}
            </div>

            {/* Section content */}
            <AnimatePresence mode="wait">
                <motion.div key={activeSection} initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -10 }} transition={{ duration: 0.2 }} className="rounded-lg bg-muted/50 p-4">
                    {stories[activeSection]?.stories?.map((story, idx) => (
                        <p key={idx} className="whitespace-pre-wrap text-sm leading-relaxed text-muted-foreground">
                            {story.storyText}
                        </p>
                    ))}
                </motion.div>
            </AnimatePresence>
        </div>
    );
}
