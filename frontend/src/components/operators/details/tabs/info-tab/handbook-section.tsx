"use client";

import { ChevronDown, FileText } from "lucide-react";
import { motion } from "motion/react";
import { useState } from "react";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "~/components/ui/motion-primitives/disclosure";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/shadcn/collapsible";
import type { HandbookItem } from "~/types/api";

interface HandbookSectionProps {
    handbook: HandbookItem;
}

export function HandbookSection({ handbook }: HandbookSectionProps) {
    const [isExpanded, setIsExpanded] = useState(false);

    if (!handbook?.storyTextAudio?.length) return null;

    return (
        <Disclosure onOpenChange={setIsExpanded} open={isExpanded}>
            <DisclosureTrigger>
                <div className="flex cursor-pointer items-center justify-between rounded-lg bg-muted/30 p-4 transition-colors hover:bg-muted/50">
                    <h3 className="font-semibold text-lg">Operator Handbook</h3>
                    <motion.div animate={{ rotate: isExpanded ? 180 : 0 }} transition={{ duration: 0.2 }}>
                        <ChevronDown className="h-5 w-5" />
                    </motion.div>
                </div>
            </DisclosureTrigger>
            <DisclosureContent>
                <div className="mt-2 space-y-4">
                    {handbook.storyTextAudio.map((storySection, sectionIndex) => (
                        <StorySection key={sectionIndex} sectionIndex={sectionIndex} storySection={storySection} />
                    ))}
                </div>
            </DisclosureContent>
        </Disclosure>
    );
}

interface StorySectionProps {
    storySection: {
        storyTitle: string;
        stories: Array<{
            storyText: string;
            unLockType?: string;
            unLockParam?: string;
            unLockString?: string;
        }>;
    };
    sectionIndex: number;
}

function StorySection({ storySection, sectionIndex }: StorySectionProps) {
    const [isOpen, setIsOpen] = useState(sectionIndex === 0);

    return (
        <Collapsible onOpenChange={setIsOpen} open={isOpen}>
            <div className="overflow-hidden rounded-lg border border-border">
                <CollapsibleTrigger asChild>
                    <div className="flex cursor-pointer items-center justify-between border-border border-b bg-card/50 p-3 transition-all hover:bg-muted/50">
                        <h4 className="font-semibold text-base">{storySection.storyTitle}</h4>
                        <motion.div animate={{ rotate: isOpen ? 180 : 0 }} transition={{ duration: 0.2 }}>
                            <ChevronDown className="h-4 w-4" />
                        </motion.div>
                    </div>
                </CollapsibleTrigger>
                <CollapsibleContent>
                    <div className="bg-card/30 p-4">
                        {storySection.stories.map((story, storyIndex) => (
                            <div className="mt-3 first:mt-0" key={storyIndex}>
                                <div className="mb-2 flex items-center text-muted-foreground text-xs">
                                    <FileText className="mr-1.5 h-3.5 w-3.5" />
                                    <span>Entry {storyIndex + 1}</span>
                                </div>
                                <div className="prose prose-sm dark:prose-invert max-w-none">
                                    {story.storyText.split("\n").map((paragraph, pIndex) => {
                                        if (!paragraph.trim()) return null;

                                        // Handle section headers with [HeaderName] format
                                        if (paragraph.startsWith("[") && paragraph.includes("]")) {
                                            const parts = paragraph.split("]");
                                            const headerText = parts[0]?.substring(1).trim() ?? "";
                                            const contentText = parts.slice(1).join("]").trim();

                                            // Stat-like headers get special formatting
                                            const isStatHeader = [
                                                "Race",
                                                "Place of Birth",
                                                "Date of Birth",
                                                "Physical Strength",
                                                "Mobility",
                                                "Physical Resilience",
                                                "Tactical Acumen",
                                                "Combat Skill",
                                                "Originium Arts Assimilation",
                                                "Height",
                                                "Weight",
                                                "Code Name",
                                                "Gender",
                                                "Combat Experience",
                                                "Cell-Originium Assimilation",
                                                "Blood Originium-Crystal Density",
                                            ].includes(headerText);

                                            return (
                                                <div className="mb-2 flex flex-wrap text-sm" key={pIndex}>
                                                    <span className="mr-2 min-w-[140px] font-bold text-primary">{headerText}:</span>
                                                    {isStatHeader ? <span className="rounded bg-muted/50 px-1.5 py-0.5 font-mono text-xs">{contentText}</span> : <span className="flex-1">{contentText}</span>}
                                                </div>
                                            );
                                        }

                                        // Technical/code-like content
                                        if (/\b(Originium|Arts|Cell|Oripathy|Protocol|System|Algorithm|Version|Module|Device|Catalyst)\b/i.test(paragraph) && paragraph.length < 100) {
                                            return (
                                                <div className="mb-2 rounded-sm border-primary/50 border-l-2 bg-muted/30 px-3 py-2" key={pIndex}>
                                                    <code className="font-mono text-xs">{paragraph}</code>
                                                </div>
                                            );
                                        }

                                        // Lists
                                        if (paragraph.trim().startsWith("-") || paragraph.trim().startsWith("•")) {
                                            return (
                                                <div className="mb-1 ml-4 flex text-sm" key={pIndex}>
                                                    <span className="mr-2 text-primary">{paragraph.trim().charAt(0)}</span>
                                                    <span>{paragraph.trim().substring(1).trim()}</span>
                                                </div>
                                            );
                                        }

                                        // Quotes
                                        if ((paragraph.trim().startsWith('"') && paragraph.trim().endsWith('"')) || (paragraph.trim().startsWith("'") && paragraph.trim().endsWith("'")) || paragraph.trim().startsWith(">")) {
                                            const quoteContent = paragraph.trim().startsWith(">")
                                                ? paragraph.trim().substring(1).trim()
                                                : paragraph
                                                      .trim()
                                                      .substring(1, paragraph.trim().length - 1)
                                                      .trim();

                                            return (
                                                <blockquote className="mb-2 rounded-r border-primary/30 border-l-4 bg-muted/20 py-1.5 pl-4 text-sm italic" key={pIndex}>
                                                    {quoteContent}
                                                </blockquote>
                                            );
                                        }

                                        // All caps emphasis
                                        if (paragraph.toUpperCase() === paragraph && paragraph.length > 10) {
                                            return (
                                                <p className="mb-2 font-semibold text-orange-600 text-sm dark:text-orange-400" key={pIndex}>
                                                    {paragraph}
                                                </p>
                                            );
                                        }

                                        // Paragraphs with numeric data
                                        if (/\b([0-9]+\.?[0-9]*%|[0-9]+[.,][0-9]+)\b/.test(paragraph) && paragraph.length < 120) {
                                            const parts = paragraph.split(/(\b[0-9]+\.?[0-9]*%|\b[0-9]+[.,][0-9]+\b)/g);
                                            return (
                                                <p className="mb-2 text-sm leading-relaxed" key={pIndex}>
                                                    {parts.map((part, partIndex) =>
                                                        /\b([0-9]+\.?[0-9]*%|[0-9]+[.,][0-9]+)\b/.test(part) ? (
                                                            <span className="font-mono font-semibold text-emerald-600 dark:text-emerald-400" key={partIndex}>
                                                                {part}
                                                            </span>
                                                        ) : (
                                                            part
                                                        ),
                                                    )}
                                                </p>
                                            );
                                        }

                                        // Regular paragraph with better typography
                                        return (
                                            <p className="mb-2 text-muted-foreground text-sm leading-relaxed" key={pIndex}>
                                                {paragraph}
                                            </p>
                                        );
                                    })}
                                </div>
                            </div>
                        ))}
                    </div>
                </CollapsibleContent>
            </div>
        </Collapsible>
    );
}
