import { Button, Toolbar, ToolbarButton, ToolbarGroup, ToolbarLink, ToolbarSeparator } from "frontend";
import { BookOpenIcon, ExternalLinkIcon, Share2Icon } from "lucide-react";

/** A link keeps its anchor semantics while joining the toolbar's roving focus. */
export const DocsLink = () => (
    <Toolbar className="w-fit" aria-label="Stage tools">
        <ToolbarButton render={<Button size="sm" variant="outline" />}>
            <Share2Icon />
            Share stage
        </ToolbarButton>
        <ToolbarSeparator />
        <ToolbarLink className="inline-flex h-8 items-center gap-1.5 rounded-lg px-2.5 font-medium text-primary text-sm underline-offset-4 hover:underline" href="https://myrtle.moe/stages/1-7">
            <BookOpenIcon className="size-4" />
            Stage guide
        </ToolbarLink>
    </Toolbar>
);

/** Rendered as a Button, the link picks up the DS's link variant. */
export const AsButtonVariant = () => (
    <Toolbar className="w-fit" aria-label="Operator resources">
        <ToolbarGroup>
            <ToolbarLink href="https://myrtle.moe/operators/mlynar" render={<Button size="sm" variant="link" />}>
                Operator page
            </ToolbarLink>
            <ToolbarLink href="https://myrtle.moe/tools/planner" render={<Button size="sm" variant="link" />}>
                Plan upgrades
            </ToolbarLink>
        </ToolbarGroup>
        <ToolbarSeparator />
        <ToolbarLink href="https://gamepress.gg" render={<Button size="sm" variant="ghost" />} rel="noreferrer" target="_blank">
            External wiki
            <ExternalLinkIcon />
        </ToolbarLink>
    </Toolbar>
);
