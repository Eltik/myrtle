"use client";

import { BookMarked, FileText, FlaskConical, ScrollText, Sparkles } from "lucide-react";
import { memo } from "react";
import type { Operator } from "~/types/api";
import { LoreSection } from "./impl/lore-section";
import { StoryEntry } from "./impl/story-entry";

interface LoreContentProps {
    operator: Operator;
}

export const LoreContent = memo(function LoreContent({ operator }: LoreContentProps) {
    const { handbook } = operator;

    const hasStories = handbook?.storyTextAudio && handbook.storyTextAudio.length > 0;

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            {/* Header */}
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl md:text-2xl">Operator Files</h2>
                <p className="mt-1 text-muted-foreground text-sm">Personal records, archives, and classified documents</p>
            </div>

            {/* Archive Files Section */}
            {hasStories && (
                <div className="space-y-4">
                    <div className="flex items-center gap-2">
                        <BookMarked className="h-5 w-5 text-primary" />
                        <h3 className="font-semibold text-foreground text-lg">Archive Files</h3>
                    </div>

                    {handbook.storyTextAudio.map((storySection, sectionIndex) => {
                        // Determine icon based on title
                        let SectionIcon = ScrollText;
                        const title = storySection.storyTitle.toLowerCase();
                        if (title.includes("clinical") || title.includes("medical")) {
                            SectionIcon = FlaskConical;
                        } else if (title.includes("promotion") || title.includes("elite")) {
                            SectionIcon = Sparkles;
                        }

                        return (
                            <LoreSection defaultOpen={sectionIndex === 0} icon={SectionIcon} key={storySection.storyTitle} title={storySection.storyTitle}>
                                {storySection.stories.map((story) => (
                                    <StoryEntry key={story.storyText.slice(0, 50)} storyText={story.storyText} />
                                ))}
                            </LoreSection>
                        );
                    })}
                </div>
            )}

            {/* Empty State */}
            {!hasStories && (
                <div className="flex flex-col items-center justify-center py-12 text-center">
                    <FileText className="mb-4 h-12 w-12 text-muted-foreground/50" />
                    <h3 className="mb-2 font-medium text-foreground">No Records Available</h3>
                    <p className="text-muted-foreground text-sm">This operator's files are classified or not yet documented.</p>
                </div>
            )}
        </div>
    );
});
