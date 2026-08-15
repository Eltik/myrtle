import { Badge, Tabs, TabsContent, TabsList, TabsTrigger } from "frontend";

// TabsContent is the shadcn-compatible alias for TabsPanel.

/** The rendered-markdown side of the changelog composer. */
export const MarkdownPreview = () => (
    <Tabs className="w-full max-w-lg gap-0 rounded-xl border bg-card" defaultValue="preview">
        <div className="flex items-center gap-2 border-border/70 border-b px-1.5 py-1.5">
            <TabsList className="h-9 p-0.5 sm:h-7">
                <TabsTrigger className="h-8 px-2.5 text-[13px] sm:h-6 sm:px-2 sm:text-[12px]" value="write">
                    Write
                </TabsTrigger>
                <TabsTrigger className="h-8 px-2.5 text-[13px] sm:h-6 sm:px-2 sm:text-[12px]" value="preview">
                    Preview
                </TabsTrigger>
            </TabsList>
        </div>
        <TabsContent className="flex flex-col gap-2 px-3 py-3" value="preview">
            <span className="font-semibold text-sm">Chapter 8 — Roaring Flare</span>
            <p className="text-muted-foreground text-sm">14 stages, 3 challenge modes. First-clear rewards include 2 Chip Catalysts.</p>
            <div className="flex gap-2">
                <Badge size="sm" variant="outline">
                    main story
                </Badge>
                <Badge size="sm" variant="outline">
                    EN
                </Badge>
            </div>
        </TabsContent>
        <TabsContent className="px-3 py-3" value="write">
            Write
        </TabsContent>
    </Tabs>
);

/** Panels carry whatever the section needs — here a stat grid. */
export const StatGrid = () => (
    <Tabs className="w-full max-w-xl" defaultValue="stats">
        <TabsList>
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="stats">Stats</TabsTrigger>
            <TabsTrigger value="skills">Skills</TabsTrigger>
        </TabsList>
        <TabsContent className="pt-5" value="stats">
            <dl className="grid grid-cols-2 gap-x-6 gap-y-2 text-sm sm:grid-cols-3">
                {[
                    { k: "HP", v: "3,468" },
                    { k: "ATK", v: "1,036" },
                    { k: "DEF", v: "455" },
                    { k: "RES", v: "0" },
                    { k: "Redeploy", v: "70s" },
                    { k: "DP cost", v: "24" },
                ].map((s) => (
                    <div className="flex items-baseline justify-between gap-3" key={s.k}>
                        <dt className="text-muted-foreground">{s.k}</dt>
                        <dd className="font-mono tabular-nums">{s.v}</dd>
                    </div>
                ))}
            </dl>
        </TabsContent>
        <TabsContent className="pt-5" value="overview">
            Overview
        </TabsContent>
        <TabsContent className="pt-5" value="skills">
            Skills
        </TabsContent>
    </Tabs>
);
