import { Badge, Collapsible, CollapsiblePanel, CollapsibleTrigger } from "frontend";
import { ChevronDown, ChevronRight, Swords } from "lucide-react";

export const BarTrigger = () => (
    <Collapsible className="flex max-w-md flex-col gap-3" defaultOpen>
        <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
            <span className="flex items-center gap-2">
                <Swords aria-hidden="true" className="size-4 text-primary" />
                <span className="font-medium text-sm">Enemy — Originium Slug</span>
            </span>
            <ChevronDown aria-hidden="true" className="size-4 rotate-180 text-muted-foreground" />
        </CollapsibleTrigger>
        <CollapsiblePanel>
            <div className="grid grid-cols-3 gap-3 rounded-lg border border-border bg-card p-4 text-sm">
                <div>
                    <p className="text-muted-foreground text-xs">HP</p>
                    <p className="font-mono tabular-nums">1,650</p>
                </div>
                <div>
                    <p className="text-muted-foreground text-xs">DEF</p>
                    <p className="font-mono tabular-nums">120</p>
                </div>
                <div>
                    <p className="text-muted-foreground text-xs">RES</p>
                    <p className="font-mono tabular-nums">0</p>
                </div>
            </div>
        </CollapsiblePanel>
    </Collapsible>
);

export const InlineTrigger = () => (
    <div className="flex max-w-md flex-col gap-3 rounded-xl border border-border bg-card p-4">
        <div>
            <p className="font-medium text-sm">Drop rates — 4-8</p>
            <p className="text-muted-foreground text-xs">Sampled from 12,480 community runs.</p>
        </div>
        <div className="flex flex-col gap-1.5 text-sm">
            <div className="flex items-center justify-between">
                <span>Manganese Ore</span>
                <span className="font-mono text-muted-foreground tabular-nums">24.6%</span>
            </div>
            <div className="flex items-center justify-between">
                <span>Polyester Pack</span>
                <span className="font-mono text-muted-foreground tabular-nums">18.4%</span>
            </div>
        </div>
        <Collapsible className="flex flex-col gap-2">
            <CollapsibleTrigger className="flex items-center gap-1.5 font-medium text-muted-foreground text-sm hover:text-foreground">
                <ChevronRight aria-hidden="true" className="size-3.5" />
                Show 7 more drops
            </CollapsibleTrigger>
            <CollapsiblePanel>
                <div className="flex flex-col gap-1.5 border-border/60 border-t pt-3 text-sm">
                    <div className="flex items-center justify-between">
                        <span>Grindstone</span>
                        <span className="font-mono text-muted-foreground tabular-nums">12.1%</span>
                    </div>
                    <div className="flex items-center justify-between">
                        <span>Orirock Cube</span>
                        <span className="font-mono text-muted-foreground tabular-nums">9.8%</span>
                    </div>
                </div>
            </CollapsiblePanel>
        </Collapsible>
    </div>
);

export const CardTrigger = () => (
    <Collapsible className="flex max-w-md flex-col gap-3" defaultOpen>
        <CollapsibleTrigger
            render={
                <button className="group flex w-full flex-col gap-3 rounded-xl border border-border bg-card p-4 text-left" type="button">
                    <span className="flex items-center justify-between">
                        <span className="font-medium text-sm">Operator development</span>
                        <Badge variant="secondary">78 / 100</Badge>
                    </span>
                    <span className="flex items-center justify-between text-muted-foreground text-xs">
                        <span>Worth 30% of your overall grade</span>
                        <ChevronDown aria-hidden="true" className="size-3.5 rotate-180" />
                    </span>
                </button>
            }
        />
        <CollapsiblePanel>
            <div className="rounded-xl border border-border bg-card p-4 text-muted-foreground text-sm">
                Raising Muelsyse to E2 60 would add an estimated <span className="font-mono text-foreground tabular-nums">+3.4</span> points.
            </div>
        </CollapsiblePanel>
    </Collapsible>
);
