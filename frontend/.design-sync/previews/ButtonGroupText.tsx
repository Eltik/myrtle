import { Button, ButtonGroup, ButtonGroupText } from "frontend";
import { ChevronLeft, ChevronRight, Minus, Plus } from "lucide-react";

export const Stepper = () => (
    <ButtonGroup>
        <Button aria-label="Lower potential" size="icon" variant="outline">
            <Minus />
        </Button>
        <ButtonGroupText>Potential 4</ButtonGroupText>
        <Button aria-label="Raise potential" size="icon" variant="outline">
            <Plus />
        </Button>
    </ButtonGroup>
);

export const Pagination = () => (
    <ButtonGroup>
        <Button aria-label="Previous page" size="icon" variant="outline">
            <ChevronLeft />
        </Button>
        <ButtonGroupText>Page 3 of 12</ButtonGroupText>
        <Button aria-label="Next page" size="icon" variant="outline">
            <ChevronRight />
        </Button>
    </ButtonGroup>
);

export const Label = () => (
    <div className="flex flex-col gap-3">
        <ButtonGroup>
            <ButtonGroupText>Server</ButtonGroupText>
            <Button variant="outline">EN</Button>
            <Button variant="outline">CN</Button>
            <Button variant="outline">JP</Button>
        </ButtonGroup>
        <ButtonGroup>
            <ButtonGroupText>Rarity</ButtonGroupText>
            <Button variant="outline">6★</Button>
            <Button variant="outline">5★</Button>
            <Button variant="outline">4★</Button>
        </ButtonGroup>
    </div>
);
