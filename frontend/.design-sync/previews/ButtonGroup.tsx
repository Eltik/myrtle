import { Button, ButtonGroup, ButtonGroupSeparator, ButtonGroupText } from "frontend";
import { ChevronDown, Copy, Download, Minus, Plus, Undo2 } from "lucide-react";

export const SplitAction = () => (
    <ButtonGroup>
        <Button>Publish tier list</Button>
        <ButtonGroupSeparator />
        <Button aria-label="More publish options" size="icon">
            <ChevronDown />
        </Button>
    </ButtonGroup>
);

export const Toolbar = () => (
    <ButtonGroup>
        <Button variant="outline">
            <Undo2 />
            Undo
        </Button>
        <Button variant="outline">
            <Copy />
            Duplicate
        </Button>
        <Button variant="outline">
            <Download />
            Export
        </Button>
    </ButtonGroup>
);

export const Stepper = () => (
    <ButtonGroup>
        <Button aria-label="Decrease promotion level" size="icon" variant="outline">
            <Minus />
        </Button>
        <ButtonGroupText>E2 Lv. 60</ButtonGroupText>
        <Button aria-label="Increase promotion level" size="icon" variant="outline">
            <Plus />
        </Button>
    </ButtonGroup>
);

export const Vertical = () => (
    <ButtonGroup orientation="vertical">
        <Button variant="outline">Roster</Button>
        <Button variant="outline">Depot</Button>
        <Button variant="outline">Base</Button>
        <Button variant="outline">Enemies</Button>
    </ButtonGroup>
);
