import { Collapsible, CollapsiblePanel, CollapsibleTrigger } from "frontend";
import { ChevronDown } from "lucide-react";

export const DropTable = () => (
    <Collapsible className="flex max-w-md flex-col gap-2" defaultOpen>
        <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3">
            <span className="font-medium text-sm">Drop table — 4-8</span>
            <ChevronDown aria-hidden="true" className="size-4 rotate-180 text-muted-foreground" />
        </CollapsibleTrigger>
        <CollapsiblePanel>
            <div className="flex flex-col gap-2 rounded-lg border border-border bg-card p-4 text-sm">
                {[
                    { name: "Manganese Ore", rate: "24.6%" },
                    { name: "Polyester Pack", rate: "18.4%" },
                    { name: "Grindstone", rate: "12.1%" },
                    { name: "Orirock Cube", rate: "9.8%" },
                ].map((drop) => (
                    <div className="flex items-center justify-between" key={drop.name}>
                        <span>{drop.name}</span>
                        <span className="font-mono text-muted-foreground tabular-nums">{drop.rate}</span>
                    </div>
                ))}
            </div>
        </CollapsiblePanel>
    </Collapsible>
);

export const BorderedPanel = () => (
    <Collapsible className="max-w-md overflow-hidden rounded-xl border border-border bg-card" defaultOpen>
        <CollapsibleTrigger className="flex w-full items-center justify-between px-4 py-3">
            <span className="font-medium text-sm">Operator notes</span>
            <ChevronDown aria-hidden="true" className="size-4 rotate-180 text-muted-foreground" />
        </CollapsibleTrigger>
        <CollapsiblePanel className="border-border/60 border-t">
            <p className="p-4 text-muted-foreground text-sm">
                Pair Młynar with a fast-redeploy Specialist so his S3 uptime lines up with the second wave. Trust him to out-heal chip damage once his talent is at potential 2.
            </p>
        </CollapsiblePanel>
    </Collapsible>
);

export const ClosedPanel = () => (
    <Collapsible className="flex max-w-md flex-col gap-2">
        <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3">
            <span className="font-medium text-sm">Riic base skills</span>
            <ChevronDown aria-hidden="true" className="size-4 text-muted-foreground" />
        </CollapsibleTrigger>
        <CollapsiblePanel>
            <div className="rounded-lg border border-border bg-card p-4 text-muted-foreground text-sm">Standardization α — Factory productivity +25%.</div>
        </CollapsiblePanel>
    </Collapsible>
);
