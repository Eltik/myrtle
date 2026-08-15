import { Button, Toggle, Toolbar, ToolbarButton, ToolbarGroup, ToolbarSeparator } from "frontend";
import { AlignCenterIcon, AlignLeftIcon, AlignRightIcon, BoldIcon, DownloadIcon, ItalicIcon, Share2Icon, UnderlineIcon } from "lucide-react";

/** Groups tighten the spacing between related controls; separators divide them. */
export const FormattingClusters = () => (
    <Toolbar className="w-fit" aria-label="Formatting">
        <ToolbarGroup>
            <ToolbarButton aria-label="Bold" render={<Toggle defaultPressed size="sm" />}>
                <BoldIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Italic" render={<Toggle size="sm" />}>
                <ItalicIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Underline" render={<Toggle size="sm" />}>
                <UnderlineIcon />
            </ToolbarButton>
        </ToolbarGroup>
        <ToolbarSeparator />
        <ToolbarGroup>
            <ToolbarButton aria-label="Align left" render={<Toggle defaultPressed size="sm" />}>
                <AlignLeftIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Align centre" render={<Toggle size="sm" />}>
                <AlignCenterIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Align right" render={<Toggle size="sm" />}>
                <AlignRightIcon />
            </ToolbarButton>
        </ToolbarGroup>
    </Toolbar>
);

/** A single group of page actions, kept together at the end of the bar. */
export const ActionCluster = () => (
    <Toolbar className="w-fit" aria-label="Tier list actions">
        <ToolbarButton render={<Button size="sm" />}>Publish</ToolbarButton>
        <ToolbarSeparator />
        <ToolbarGroup>
            <ToolbarButton render={<Button size="sm" variant="outline" />}>
                <Share2Icon />
                Share
            </ToolbarButton>
            <ToolbarButton render={<Button size="sm" variant="outline" />}>
                <DownloadIcon />
                Export
            </ToolbarButton>
        </ToolbarGroup>
    </Toolbar>
);

/** Stacked groups in a vertical toolbar. */
export const Vertical = () => (
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
            <ToolbarButton aria-label="Align left" render={<Toggle defaultPressed size="sm" />}>
                <AlignLeftIcon />
            </ToolbarButton>
            <ToolbarButton aria-label="Align centre" render={<Toggle size="sm" />}>
                <AlignCenterIcon />
            </ToolbarButton>
        </ToolbarGroup>
    </Toolbar>
);
