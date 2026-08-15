import { Button } from "frontend";
import { DownloadIcon, PlusIcon, Trash2Icon } from "lucide-react";

export const Variants = () => (
    <div className="flex flex-wrap items-center gap-2">
        <Button>Save changes</Button>
        <Button variant="secondary">Secondary</Button>
        <Button variant="outline">Outline</Button>
        <Button variant="ghost">Ghost</Button>
        <Button variant="link">Link</Button>
        <Button variant="destructive">Delete list</Button>
        <Button variant="destructive-outline">Remove</Button>
    </div>
);

export const Sizes = () => (
    <div className="flex flex-wrap items-center gap-2">
        <Button size="xs">Extra small</Button>
        <Button size="sm">Small</Button>
        <Button size="default">Default</Button>
        <Button size="lg">Large</Button>
        <Button size="xl">Extra large</Button>
    </div>
);

export const WithIcons = () => (
    <div className="flex flex-wrap items-center gap-2">
        <Button>
            <PlusIcon />
            New tier list
        </Button>
        <Button variant="outline">
            <DownloadIcon />
            Export
        </Button>
        <Button size="icon" variant="outline" aria-label="Add operator">
            <PlusIcon />
        </Button>
        <Button size="icon" variant="destructive-outline" aria-label="Delete">
            <Trash2Icon />
        </Button>
    </div>
);

export const States = () => (
    <div className="flex flex-wrap items-center gap-2">
        <Button loading>Publishing</Button>
        <Button disabled>Disabled</Button>
        <Button variant="outline" disabled>
            Disabled outline
        </Button>
        <Button variant="secondary" loading>
            Loading
        </Button>
    </div>
);
