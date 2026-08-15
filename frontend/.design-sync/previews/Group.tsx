import { Button, Group, GroupSeparator, GroupText, Input } from "frontend";
import { LayoutGrid, List, RotateCcw, Rows3, Share2, Star } from "lucide-react";

export const SegmentedButtons = () => (
    <Group>
        <Button variant="outline">All stages</Button>
        <Button variant="outline">Cleared</Button>
        <Button variant="outline">Missing 3★</Button>
    </Group>
);

export const IconToolbar = () => (
    <Group>
        <Button aria-label="Grid view" size="icon" variant="outline">
            <LayoutGrid />
        </Button>
        <Button aria-label="Row view" size="icon" variant="outline">
            <Rows3 />
        </Button>
        <Button aria-label="List view" size="icon" variant="outline">
            <List />
        </Button>
        <GroupSeparator />
        <Button aria-label="Reset filters" size="icon" variant="outline">
            <RotateCcw />
        </Button>
        <Button aria-label="Share tier list" size="icon" variant="outline">
            <Share2 />
        </Button>
    </Group>
);

export const WithTextAndInput = () => (
    <Group>
        <GroupText>myrtle.moe/u/</GroupText>
        <Input className="w-56" defaultValue="dr-kaltsit" />
        <Button variant="outline">
            <Star />
            Claim
        </Button>
    </Group>
);

export const Vertical = () => (
    <Group orientation="vertical">
        <Button variant="outline">Export as image</Button>
        <Button variant="outline">Copy share link</Button>
        <Button variant="outline">Duplicate list</Button>
        <GroupSeparator orientation="horizontal" />
        <Button variant="destructive-outline">Delete list</Button>
    </Group>
);
