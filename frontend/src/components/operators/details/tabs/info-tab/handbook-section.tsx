"use client";

import { useState } from "react";
import { ChevronDown } from "lucide-react";
import { motion } from "motion/react";
import type { HandbookItem } from "~/types/api";
import { Disclosure, DisclosureTrigger, DisclosureContent } from "~/components/ui/motion-primitives/disclosure";

interface HandbookSectionProps {
    handbook: HandbookItem;
}

export function HandbookSection({ handbook }: HandbookSectionProps) {
    const [isExpanded, setIsExpanded] = useState(false);

    const sections = [
        { title: "Basic Info", content: handbook.storyTextAudio?.[0]?.stories?.[0]?.storyText },
        ...(handbook.storyTextAudio?.slice(1).map((section) => ({
            title: section.storyTitle,
            content: section.stories?.[0]?.storyText,
        })) ?? []),
    ].filter((s) => s.content);

    if (sections.length === 0) return null;

    return (
        <Disclosure open={isExpanded} onOpenChange={setIsExpanded}>
            <DisclosureTrigger>
                <div className="flex cursor-pointer items-center justify-between rounded-lg bg-muted/30 p-4 transition-colors hover:bg-muted/50">
                    <h3 className="text-lg font-semibold">Operator Handbook</h3>
                    <motion.div animate={{ rotate: isExpanded ? 180 : 0 }} transition={{ duration: 0.2 }}>
                        <ChevronDown className="h-5 w-5" />
                    </motion.div>
                </div>
            </DisclosureTrigger>
            <DisclosureContent>
                <div className="mt-2 space-y-4">
                    {sections.map((section, index) => (
                        <div key={index} className="rounded-lg border border-border bg-card/50 p-4">
                            <h4 className="mb-2 font-semibold text-primary">{section.title}</h4>
                            <p className="whitespace-pre-wrap text-sm leading-relaxed text-muted-foreground">{section.content}</p>
                        </div>
                    ))}
                </div>
            </DisclosureContent>
        </Disclosure>
    );
}
