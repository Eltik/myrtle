import { Button, Toggle, Toolbar, ToolbarButton, ToolbarGroup, ToolbarSeparator } from "frontend";
import { BoldIcon, DownloadIcon, ItalicIcon, ListIcon, Share2Icon, Trash2Icon } from "lucide-react";

/** Toolbar buttons take a `render` element, so any Button or Toggle variant fits. */
export const Actions = () => (
    <Toolbar className="w-fit" aria-label="Tier list actions">
        <ToolbarButton render={<Button size="sm" />}>Publish</ToolbarButton>
        <ToolbarButton render={<Button size="sm" variant="outline" />}>
            <Share2Icon />
            Share
        </ToolbarButton>
        <ToolbarButton render={<Button size="sm" variant="outline" />}>
            <DownloadIcon />
            Export
        </ToolbarButton>
        <ToolbarSeparator />
        <ToolbarButton aria-label="Delete list" render={<Button size="icon-sm" variant="destructive-outline" />}>
            <Trash2Icon />
        </ToolbarButton>
    </Toolbar>
);

/** Rendered as toggles, the buttons carry a pressed state. */
export const PressedState = () => (
    <Toolbar className="w-fit" aria-label="Formatting">
        <ToolbarGroup>
            <ToolbarButton aria-label="Bold" render={<Toggle defaultPressed size="sm" />}>
                <BoldIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Italic" render={<Toggle size="sm" />}>
                <ItalicIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Bullet list" render={<Toggle size="sm" />}>
                <ListIcon />
            </ToolbarButton>
        </ToolbarGroup>
    </Toolbar>
);

/** Disabled — publishing needs at least one operator placed. */
export const DisabledAction = () => (
    <Toolbar className="w-fit" aria-label="Tier list actions">
        <ToolbarButton disabled render={<Button size="sm" />}>
            Publish
        </ToolbarButton>
        <ToolbarButton disabled render={<Button size="sm" variant="outline" />}>
            <Share2Icon />
            Share
        </ToolbarButton>
        <ToolbarSeparator />
        <ToolbarButton render={<Button size="sm" variant="outline" />}>Save draft</ToolbarButton>
    </Toolbar>
);
