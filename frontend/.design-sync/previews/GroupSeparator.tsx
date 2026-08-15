import { Button, Group, GroupSeparator } from "frontend";
import { Bold, Italic, Link2, ListOrdered, Redo2, Undo2 } from "lucide-react";

export const BetweenClusters = () => (
    <Group>
        <Button aria-label="Undo" size="icon" variant="outline">
            <Undo2 />
        </Button>
        <Button aria-label="Redo" size="icon" variant="outline">
            <Redo2 />
        </Button>
        <GroupSeparator />
        <Button aria-label="Bold" size="icon" variant="outline">
            <Bold />
        </Button>
        <Button aria-label="Italic" size="icon" variant="outline">
            <Italic />
        </Button>
        <Button aria-label="Ordered list" size="icon" variant="outline">
            <ListOrdered />
        </Button>
        <GroupSeparator />
        <Button aria-label="Insert link" size="icon" variant="outline">
            <Link2 />
        </Button>
    </Group>
);

export const SplitButton = () => (
    <Group>
        <Button>Save operator note</Button>
        <GroupSeparator />
        <Button variant="default">Save & publish</Button>
    </Group>
);

export const Vertical = () => (
    <Group orientation="vertical">
        <Button variant="outline">Copy share link</Button>
        <Button variant="outline">Export as image</Button>
        <GroupSeparator orientation="horizontal" />
        <Button variant="destructive-outline">Delete list</Button>
    </Group>
);
