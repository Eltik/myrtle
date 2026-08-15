import { Badge, Collapsible, CollapsibleContent, CollapsibleTrigger } from "frontend";
import { ChevronDown } from "lucide-react";

// CollapsibleContent is the shadcn-compatible alias for CollapsiblePanel.

export const AppearsIn = () => (
    <Collapsible className="flex max-w-md flex-col gap-2" defaultOpen>
        <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3">
            <span className="flex items-center gap-2">
                <span className="font-medium text-sm">Appears in</span>
                <Badge className="text-[10px]" variant="outline">
                    5
                </Badge>
            </span>
            <ChevronDown aria-hidden="true" className="size-4 rotate-180 text-muted-foreground" />
        </CollapsibleTrigger>
        <CollapsibleContent>
            <ul className="flex flex-col gap-1.5 rounded-lg border border-border bg-card p-4 text-sm">
                {["1-7", "4-8", "S4-1", "CE-6", "9-16"].map((code) => (
                    <li className="flex items-center justify-between" key={code}>
                        <span className="font-mono">{code}</span>
                        <span className="text-muted-foreground text-xs">Chapter stage</span>
                    </li>
                ))}
            </ul>
        </CollapsibleContent>
    </Collapsible>
);

export const LoreEntry = () => (
    <Collapsible className="flex max-w-md flex-col gap-2" defaultOpen>
        <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3">
            <span className="font-medium text-sm">Archive file 2</span>
            <ChevronDown aria-hidden="true" className="size-4 rotate-180 text-muted-foreground" />
        </CollapsibleTrigger>
        <CollapsibleContent>
            <p className="rounded-lg border border-border bg-card p-4 text-muted-foreground text-sm">
                Rhine Lab's Ecological Section reports that Director Muelsyse has again bypassed containment protocol. Recommend continued observation; results remain
                unreproducible outside her presence.
            </p>
        </CollapsibleContent>
    </Collapsible>
);

export const CollapsedByDefault = () => (
    <Collapsible className="flex max-w-md flex-col gap-2">
        <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3">
            <span className="font-medium text-sm">Trust stat bonuses</span>
            <ChevronDown aria-hidden="true" className="size-4 text-muted-foreground" />
        </CollapsibleTrigger>
        <CollapsibleContent>
            <div className="rounded-lg border border-border bg-card p-4 text-sm">+90 ATK, +150 HP at 200% trust.</div>
        </CollapsibleContent>
    </Collapsible>
);
