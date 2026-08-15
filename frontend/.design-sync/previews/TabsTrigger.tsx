import { Tabs, TabsContent, TabsList, TabsTrigger } from "frontend";
import { EyeIcon, PencilLineIcon } from "lucide-react";

// TabsTrigger is the shadcn-compatible alias for TabsTab; TabsContent aliases TabsPanel.

/** Ported from the markdown editor: a compact Write / Preview switch. */
export const WriteAndPreview = () => (
    <Tabs className="w-full max-w-lg gap-0 rounded-xl border bg-card" defaultValue="write">
        <div className="flex flex-wrap items-center justify-between gap-x-2 gap-y-1 border-border/70 border-b px-1.5 py-1.5">
            <TabsList className="h-9 p-0.5 sm:h-7" variant="default">
                <TabsTrigger className="h-8 gap-1.5 px-2.5 text-[13px] sm:h-6 sm:px-2 sm:text-[12px]" value="write">
                    <PencilLineIcon className="size-4 sm:size-3.5" />
                    <span>Write</span>
                </TabsTrigger>
                <TabsTrigger className="h-8 gap-1.5 px-2.5 text-[13px] sm:h-6 sm:px-2 sm:text-[12px]" value="preview">
                    <EyeIcon className="size-4 sm:size-3.5" />
                    <span>Preview</span>
                </TabsTrigger>
            </TabsList>
            <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">184 / 2000</span>
        </div>
        <TabsContent className="px-3 py-3 text-sm" value="write">
            <span className="text-muted-foreground">Młynar clears Chapter 8 faster than any other Guard, but he needs </span>
            <span className="font-mono text-[13px]">**12 Bipolar Nanoflake**</span>
            <span className="text-muted-foreground"> to reach M3.</span>
        </TabsContent>
        <TabsContent className="px-3 py-3 text-sm" value="preview">
            Preview
        </TabsContent>
    </Tabs>
);

/** The same alias driving a page-level section switch. */
export const SectionSwitch = () => (
    <Tabs className="w-full max-w-xl" defaultValue="ranked">
        <TabsList>
            <TabsTrigger value="ranked">Ranked</TabsTrigger>
            <TabsTrigger value="movers">Biggest movers</TabsTrigger>
            <TabsTrigger value="you">Your standing</TabsTrigger>
        </TabsList>
        <TabsContent className="pt-4 text-muted-foreground text-sm" value="ranked">
            Showing 1-25 of 48,102 Doctors, sorted by stage completion.
        </TabsContent>
        <TabsContent className="pt-4" value="movers">
            Biggest movers
        </TabsContent>
        <TabsContent className="pt-4" value="you">
            Your standing
        </TabsContent>
    </Tabs>
);
