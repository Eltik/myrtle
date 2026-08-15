import { Badge, Collapsible, CollapsiblePanel, CollapsibleTrigger } from "frontend";
import { ChevronDown, Sparkles } from "lucide-react";

export const SummonsSection = () => (
    <Collapsible className="flex max-w-md flex-col gap-3" defaultOpen>
        <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
            <span className="flex items-center gap-2">
                <Sparkles aria-hidden="true" className="size-4 text-primary" />
                <span className="font-medium text-sm">Summons</span>
                <Badge className="text-[10px]" variant="outline">
                    1
                </Badge>
            </span>
            <ChevronDown aria-hidden="true" className="size-4 rotate-180" />
        </CollapsibleTrigger>
        <CollapsiblePanel>
            <div className="flex flex-col gap-2 rounded-lg border border-border bg-card p-4">
                <div className="flex items-center justify-between text-sm">
                    <span className="font-medium">Elemental Duplicate</span>
                    <span className="font-mono text-muted-foreground tabular-nums">E2 90</span>
                </div>
                <p className="text-muted-foreground text-xs">Copies 80% of the source operator's ATK and HP. Blocks 1 enemy; expires after 25 seconds.</p>
            </div>
        </CollapsiblePanel>
    </Collapsible>
);

export const Collapsed = () => (
    <Collapsible className="flex max-w-md flex-col gap-3">
        <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
            <span className="font-medium text-sm">Base skills</span>
            <ChevronDown aria-hidden="true" className="size-4 text-muted-foreground" />
        </CollapsibleTrigger>
        <CollapsiblePanel>
            <div className="rounded-lg border border-border bg-card p-4 text-muted-foreground text-sm">Order Reception β — order acquisition efficiency +30%.</div>
        </CollapsiblePanel>
    </Collapsible>
);

export const StackedSections = () => (
    <div className="flex max-w-md flex-col gap-2">
        <Collapsible className="flex flex-col gap-2" defaultOpen>
            <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3">
                <span className="font-medium text-sm">Level-up cost</span>
                <ChevronDown aria-hidden="true" className="size-4 rotate-180 text-muted-foreground" />
            </CollapsibleTrigger>
            <CollapsiblePanel>
                <div className="flex flex-col gap-1.5 rounded-lg border border-border bg-card p-4 text-sm">
                    <div className="flex items-center justify-between">
                        <span className="text-muted-foreground">LMD</span>
                        <span className="font-mono tabular-nums">1,116,000</span>
                    </div>
                    <div className="flex items-center justify-between">
                        <span className="text-muted-foreground">EXP</span>
                        <span className="font-mono tabular-nums">1,048,320</span>
                    </div>
                </div>
            </CollapsiblePanel>
        </Collapsible>
        <Collapsible className="flex flex-col gap-2">
            <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3">
                <span className="font-medium text-sm">Skill masteries</span>
                <ChevronDown aria-hidden="true" className="size-4 text-muted-foreground" />
            </CollapsibleTrigger>
            <CollapsiblePanel>
                <div className="rounded-lg border border-border bg-card p-4 text-sm">S3 M3 needs 8 Bipolar Nanoflake.</div>
            </CollapsiblePanel>
        </Collapsible>
    </div>
);
