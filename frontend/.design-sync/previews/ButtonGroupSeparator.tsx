import { Button, ButtonGroup, ButtonGroupSeparator } from "frontend";
import { ChevronDown, Filter, RotateCcw, Share2 } from "lucide-react";

export const SplitAction = () => (
    <ButtonGroup>
        <Button>Run optimiser</Button>
        <ButtonGroupSeparator />
        <Button aria-label="More optimiser options" size="icon">
            <ChevronDown />
        </Button>
    </ButtonGroup>
);

export const SeparatedToolbar = () => (
    <ButtonGroup>
        <Button variant="outline">All operators</Button>
        <Button variant="outline">Owned</Button>
        <Button variant="outline">Missing</Button>
        <ButtonGroupSeparator />
        <Button aria-label="Filter" size="icon" variant="outline">
            <Filter />
        </Button>
        <Button aria-label="Reset" size="icon" variant="outline">
            <RotateCcw />
        </Button>
        <Button aria-label="Share" size="icon" variant="outline">
            <Share2 />
        </Button>
    </ButtonGroup>
);

export const Vertical = () => (
    <ButtonGroup orientation="vertical">
        <Button variant="outline">Sync roster</Button>
        <Button variant="outline">Sync depot</Button>
        <ButtonGroupSeparator orientation="horizontal" />
        <Button variant="destructive-outline">Unlink account</Button>
    </ButtonGroup>
);
