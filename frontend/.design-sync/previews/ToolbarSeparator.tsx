import { Button, Input, Toggle, Toolbar, ToolbarButton, ToolbarGroup, ToolbarInput, ToolbarSeparator } from "frontend";
import { BoldIcon, EyeIcon, ItalicIcon, ListIcon, QuoteIcon } from "lucide-react";

/** Vertical by default — a hairline between two clusters, stretched to the bar's height. */
export const BetweenGroups = () => (
    <Toolbar className="w-fit" aria-label="Formatting">
        <ToolbarGroup>
            <ToolbarButton aria-label="Bold" render={<Toggle defaultPressed size="sm" />}>
                <BoldIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Italic" render={<Toggle size="sm" />}>
                <ItalicIcon />
            </ToolbarButton>
        </ToolbarGroup>
        <ToolbarSeparator />
        <ToolbarGroup>
            <ToolbarButton aria-label="Bullet list" render={<Toggle size="sm" />}>
                <ListIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Quote" render={<Toggle size="sm" />}>
                <QuoteIcon />
            </ToolbarButton>
        </ToolbarGroup>
        <ToolbarSeparator />
        <ToolbarButton render={<Button size="sm" variant="outline" />}>
            <EyeIcon />
            Preview
        </ToolbarButton>
    </Toolbar>
);

/** Dividing a search field from the actions it applies to. */
export const AroundInput = () => (
    <Toolbar className="w-fit" aria-label="Roster">
        <ToolbarInput render={<Input className="w-56" placeholder="Search operators…" size="sm" type="search" />} />
        <ToolbarSeparator />
        <ToolbarButton render={<Button size="sm" variant="ghost" />}>Clear</ToolbarButton>
        <ToolbarSeparator />
        <ToolbarButton render={<Button size="sm" variant="outline" />}>Save view</ToolbarButton>
    </Toolbar>
);

/** In a vertical toolbar the separator turns into a full-width rule. */
export const Horizontal = () => (
    <Toolbar aria-label="Formatting" className="w-fit flex-col" orientation="vertical">
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
