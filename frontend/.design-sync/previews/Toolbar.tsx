import { Button, Input, Toggle, Toolbar, ToolbarButton, ToolbarGroup, ToolbarInput, ToolbarSeparator } from "frontend";
import { BoldIcon, CodeIcon, EyeIcon, ItalicIcon, LinkIcon, ListIcon, ListOrderedIcon, QuoteIcon, RedoIcon, UndoIcon } from "lucide-react";

/** The markdown editor's formatting bar, rebuilt on the Toolbar primitives. */
export const Formatting = () => (
    <Toolbar className="w-fit" aria-label="Formatting">
        <ToolbarGroup>
            <ToolbarButton aria-label="Bold" render={<Toggle defaultPressed size="sm" />}>
                <BoldIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Italic" render={<Toggle size="sm" />}>
                <ItalicIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Inline code" render={<Toggle size="sm" />}>
                <CodeIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Link" render={<Toggle size="sm" />}>
                <LinkIcon />
            </ToolbarButton>
        </ToolbarGroup>
        <ToolbarSeparator />
        <ToolbarGroup>
            <ToolbarButton aria-label="Bullet list" render={<Toggle size="sm" />}>
                <ListIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Numbered list" render={<Toggle size="sm" />}>
                <ListOrderedIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Quote" render={<Toggle size="sm" />}>
                <QuoteIcon />
            </ToolbarButton>
        </ToolbarGroup>
    </Toolbar>
);

/** With a filter field and a trailing action — the roster's table toolbar. */
export const WithInput = () => (
    <Toolbar className="w-fit" aria-label="Roster">
        <ToolbarInput render={<Input className="w-56" placeholder="Filter operators…" size="sm" type="search" />} />
        <ToolbarSeparator />
        <ToolbarGroup>
            <ToolbarButton aria-label="Undo" render={<Button size="icon-sm" variant="ghost" />}>
                <UndoIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Redo" render={<Button size="icon-sm" variant="ghost" />}>
                <RedoIcon />
            </ToolbarButton>
        </ToolbarGroup>
        <ToolbarSeparator />
        <ToolbarButton render={<Button size="sm" variant="outline" />}>
            <EyeIcon />
            Preview
        </ToolbarButton>
    </Toolbar>
);

/** `orientation="vertical"` stacks the groups and turns the separator horizontal. */
export const Vertical = () => (
    <Toolbar aria-label="Canvas tools" className="w-fit flex-col" orientation="vertical">
        <ToolbarGroup className="flex-col">
            <ToolbarButton aria-label="Bold" render={<Toggle defaultPressed size="sm" />}>
                <BoldIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Italic" render={<Toggle size="sm" />}>
                <ItalicIcon />
            </ToolbarButton>
        </ToolbarGroup>
        <ToolbarSeparator />
        <ToolbarGroup className="flex-col">
            <ToolbarButton aria-label="Bullet list" render={<Toggle size="sm" />}>
                <ListIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Quote" render={<Toggle size="sm" />}>
                <QuoteIcon />
            </ToolbarButton>
        </ToolbarGroup>
    </Toolbar>
);
