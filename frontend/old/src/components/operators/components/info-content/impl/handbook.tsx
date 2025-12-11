import { ChevronDown, FileText } from "lucide-react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/collapsible";
import type { Operator } from "~/types/impl/api/static/operator";

export const Handbook = ({ operator, isHandbookExpanded, setIsHandbookExpanded }: { operator: Operator; isHandbookExpanded: boolean; setIsHandbookExpanded: (isHandbookExpanded: boolean) => void }) => {
    return (
        <>
            {operator.handbook && (
                <div className="mt-4">
                    <Collapsible defaultOpen={isHandbookExpanded} onOpenChange={() => setIsHandbookExpanded(!isHandbookExpanded)}>
                        <CollapsibleTrigger asChild>
                            <div className="flex cursor-pointer flex-row items-center rounded-md px-2 py-1 transition-all duration-150 hover:bg-primary-foreground">
                                <h2 className="font-bold text-lg">Operator Files</h2>
                                <ChevronDown className={`ml-auto h-5 w-5 transition-transform ${isHandbookExpanded ? "rotate-180" : ""}`} />
                            </div>
                        </CollapsibleTrigger>
                        <CollapsibleContent>
                            <div className="mt-2 space-y-4">
                                {operator.handbook.storyTextAudio.map((storySection, sectionIndex) => (
                                    <Collapsible defaultOpen={true} key={sectionIndex}>
                                        <div className="rounded-md border">
                                            <CollapsibleTrigger asChild>
                                                <div className="flex cursor-pointer flex-row items-center border-b p-3 transition-all hover:bg-muted/50">
                                                    <h3 className="font-semibold text-base">{storySection.storyTitle}</h3>
                                                    <ChevronDown className="ml-auto h-4 w-4" />
                                                </div>
                                            </CollapsibleTrigger>
                                            <CollapsibleContent>
                                                <div className="p-4">
                                                    {storySection.stories.map((story, storyIndex) => (
                                                        <div className="mt-3 first:mt-0" key={storyIndex}>
                                                            <div className="mb-2 flex items-center text-muted-foreground text-xs">
                                                                <FileText className="mr-1.5 h-3.5 w-3.5" />
                                                                <span>Entry {sectionIndex + 1}</span>
                                                            </div>
                                                            <div className="prose prose-xs dark:prose-invert max-w-none text-sm">
                                                                {story.storyText.split("\n").map((paragraph, pIndex) => {
                                                                    // Skip empty paragraphs
                                                                    if (!paragraph.trim()) return null;

                                                                    // Handle section headers with [HeaderName] format
                                                                    if (paragraph.startsWith("[") && paragraph.includes("]")) {
                                                                        const parts = paragraph.split("]");
                                                                        const headerText = parts[0]?.substring(1).trim() ?? "";
                                                                        const contentText = parts.slice(1).join("]").trim();

                                                                        // Style content text based on header type
                                                                        const formattedContent = (() => {
                                                                            // For headers related to stats or measurements, use monospace
                                                                            if (
                                                                                [
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
                                                                                ].includes(headerText)
                                                                            ) {
                                                                                return <span className="rounded bg-muted/50 px-1.5 py-0.5 font-mono">{contentText}</span>;
                                                                            }

                                                                            // Default to regular formatting
                                                                            return <span className="flex-1">{contentText}</span>;
                                                                        })();

                                                                        return (
                                                                            <div className="mb-2 flex text-xs md:text-sm" key={pIndex}>
                                                                                <span className="mr-2 min-w-[140px] font-bold text-primary">{headerText}:</span>
                                                                                {formattedContent}
                                                                            </div>
                                                                        );
                                                                    }

                                                                    // Handle technical or code-like content (contains technical terms or specific patterns)
                                                                    if (/\b(Originium|Arts|Cell|Oripathy|Protocol|System|Algorithm|Version|Module|Device|Catalyst)\b/i.test(paragraph) && paragraph.length < 100) {
                                                                        return (
                                                                            <div className="mb-2 rounded-sm border-primary/50 border-l-2 bg-muted/30 px-3 py-2" key={pIndex}>
                                                                                <code className="font-mono text-xs md:text-sm">{paragraph}</code>
                                                                            </div>
                                                                        );
                                                                    }

                                                                    // Handle lists (lines starting with "-" or "•")
                                                                    if (paragraph.trim().startsWith("-") || paragraph.trim().startsWith("•")) {
                                                                        return (
                                                                            <div className="mb-1 ml-4 flex text-xs md:text-sm" key={pIndex}>
                                                                                <span className="mr-2 text-primary">{paragraph.trim().charAt(0)}</span>
                                                                                <span>{paragraph.trim().substring(1).trim()}</span>
                                                                            </div>
                                                                        );
                                                                    }

                                                                    // Handle quotes (wrapped in quotes or starting with >)
                                                                    if ((paragraph.trim().startsWith('"') && paragraph.trim().endsWith('"')) || (paragraph.trim().startsWith("'") && paragraph.trim().endsWith("'")) || paragraph.trim().startsWith(">")) {
                                                                        const quoteContent = paragraph.trim().startsWith(">")
                                                                            ? paragraph.trim().substring(1).trim()
                                                                            : paragraph
                                                                                  .trim()
                                                                                  .substring(1, paragraph.trim().length - 1)
                                                                                  .trim();

                                                                        return (
                                                                            <blockquote className="mb-2 rounded-r border-primary/30 border-l-4 bg-muted/20 py-1.5 pl-4 text-xs italic md:text-sm" key={pIndex}>
                                                                                {quoteContent}
                                                                            </blockquote>
                                                                        );
                                                                    }

                                                                    // Handle emphasized paragraphs (all caps sections or exclamation marks)
                                                                    if (paragraph.toUpperCase() === paragraph && paragraph.length > 10) {
                                                                        return (
                                                                            <p className="mb-2 font-semibold text-orange-600 text-xs md:text-sm dark:text-orange-400" key={pIndex}>
                                                                                {paragraph}
                                                                            </p>
                                                                        );
                                                                    }

                                                                    // Handle paragraphs with numeric data or statistics
                                                                    if (/\b([0-9]+\.?[0-9]*%|[0-9]+[.,][0-9]+)\b/.test(paragraph) && paragraph.length < 120) {
                                                                        // Replace numeric values with styled spans
                                                                        const parts = paragraph.split(/(\b[0-9]+\.?[0-9]*%|\b[0-9]+[.,][0-9]+\b)/g);

                                                                        return (
                                                                            <p className="mb-2 text-xs md:text-sm" key={pIndex}>
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

                                                                    // Regular paragraph with better spacing
                                                                    return (
                                                                        <p className="mb-2 text-xs md:text-sm" key={pIndex}>
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
                                ))}
                            </div>
                        </CollapsibleContent>
                    </Collapsible>
                </div>
            )}
        </>
    );
};
