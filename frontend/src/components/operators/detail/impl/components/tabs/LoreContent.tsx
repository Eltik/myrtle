import { BookMarked, ChevronDown, FileText, FlaskConical, ScrollText, Sparkles } from "lucide-react";
import { memo, useState } from "react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { cn } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";

interface ILoreContentProps {
    operator: IOperatorListItem;
}

const STAT_HEADERS = new Set([
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
    "Infection Status",
]);

export const LoreContent = memo(function LoreContent({ operator }: ILoreContentProps) {
    const stories = operator.handbook?.storyTextAudio ?? [];
    const hasStories = stories.length > 0;

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl md:text-2xl">Operator Files</h2>
                <p className="mt-1 text-muted-foreground text-sm">Personal records, archives, and classified documents.</p>
            </div>

            {hasStories ? (
                <div className="space-y-4">
                    <div className="flex items-center gap-2">
                        <BookMarked className="h-5 w-5 text-primary" />
                        <h3 className="font-semibold text-foreground text-lg">Archive Files</h3>
                    </div>
                    {stories.map((section, idx) => {
                        const title = section.storyTitle.toLowerCase();
                        let Icon: React.ElementType = ScrollText;
                        if (title.includes("clinical") || title.includes("medical")) Icon = FlaskConical;
                        else if (title.includes("promotion") || title.includes("elite")) Icon = Sparkles;
                        return (
                            <LoreSection key={section.storyTitle} title={section.storyTitle} icon={Icon} defaultOpen={idx === 0}>
                                {section.stories.map((story) => (
                                    <StoryEntry key={story.storyText.slice(0, 50)} text={story.storyText} />
                                ))}
                            </LoreSection>
                        );
                    })}
                </div>
            ) : (
                <div className="flex flex-col items-center justify-center py-12 text-center">
                    <FileText className="mb-4 h-12 w-12 text-muted-foreground/50" />
                    <h3 className="mb-2 font-medium text-foreground">No Records Available</h3>
                    <p className="text-muted-foreground text-sm">This operator's files are classified or not yet documented.</p>
                </div>
            )}
        </div>
    );
});

function LoreSection({ title, icon: Icon, children, defaultOpen = false }: { title: string; icon: React.ElementType; children: React.ReactNode; defaultOpen?: boolean }) {
    const [open, setOpen] = useState(defaultOpen);
    return (
        <Collapsible open={open} onOpenChange={setOpen}>
            <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                <span className="flex items-center gap-3">
                    <Icon className="h-4 w-4 text-primary" />
                    <span className="font-medium text-sm">{title}</span>
                </span>
                <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform", open && "rotate-180")} />
            </CollapsibleTrigger>
            <CollapsibleContent>
                <div className="mt-3 rounded-lg border border-border/50 bg-card/30 p-4">{children}</div>
            </CollapsibleContent>
        </Collapsible>
    );
}

function StoryEntry({ text }: { text: string }) {
    const paragraphs = text.split("\n");
    return <div className="prose-sm max-w-none">{paragraphs.map((p, idx) => renderParagraph(p, idx))}</div>;
}

function renderParagraph(paragraph: string, idx: number) {
    const trimmed = paragraph.trim();
    if (!trimmed) return null;

    if (trimmed.startsWith("[") && trimmed.includes("]")) {
        const close = trimmed.indexOf("]");
        const header = trimmed.substring(1, close).trim();
        const content = trimmed.substring(close + 1).trim();
        const isStat = STAT_HEADERS.has(header);
        return (
            <div className="flex flex-col gap-1 py-1.5 sm:flex-row sm:items-baseline sm:gap-3" key={`hdr-${idx}`}>
                <span className="min-w-40 shrink-0 font-semibold text-primary text-xs uppercase tracking-wide sm:text-right">{header}</span>
                <span className={cn("flex-1 text-foreground text-sm leading-relaxed", isStat && "rounded-md bg-secondary/40 px-2 py-1 font-mono text-xs")}>{content || "-"}</span>
            </div>
        );
    }

    if (/^[-•·]/.test(trimmed)) {
        return (
            <div className="flex gap-3 py-1 pl-2" key={`bul-${idx}`}>
                <span className="mt-1 text-primary">•</span>
                <span className="flex-1 text-muted-foreground text-sm leading-relaxed">{trimmed.substring(1).trim()}</span>
            </div>
        );
    }

    if ((trimmed.startsWith('"') && trimmed.endsWith('"')) || trimmed.startsWith(">") || trimmed.startsWith("「") || trimmed.startsWith("『")) {
        let q = trimmed;
        if (trimmed.startsWith(">")) q = trimmed.substring(1).trim();
        else if (trimmed.startsWith('"') && trimmed.endsWith('"')) q = trimmed.substring(1, trimmed.length - 1).trim();
        return (
            <blockquote className="my-3 rounded-r-lg border-primary/40 border-l-4 bg-secondary/30 py-3 pr-4 pl-4" key={`q-${idx}`}>
                <p className="text-muted-foreground text-sm italic leading-relaxed">{q}</p>
            </blockquote>
        );
    }

    return (
        <p className="my-2 text-muted-foreground text-sm leading-relaxed" key={`p-${idx}`}>
            {trimmed}
        </p>
    );
}
