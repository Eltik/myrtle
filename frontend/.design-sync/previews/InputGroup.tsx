import { Badge, Button, InputGroup, InputGroupAddon, InputGroupInput, InputGroupText, InputGroupTextarea, Kbd } from "frontend";
import { Search, Send, Zap, X } from "lucide-react";

export const SearchField = () => (
    <InputGroup className="max-w-sm">
        <InputGroupAddon>
            <Search aria-hidden="true" />
        </InputGroupAddon>
        <InputGroupInput aria-label="Search doctors" defaultValue="Kal'tsit" placeholder="Search by nickname…" />
        <InputGroupAddon align="inline-end">
            <Button aria-label="Clear search" size="icon-xs" variant="ghost">
                <X aria-hidden="true" />
            </Button>
        </InputGroupAddon>
    </InputGroup>
);

export const WithShortcut = () => (
    <InputGroup className="max-w-sm">
        <InputGroupAddon>
            <Search aria-hidden="true" />
        </InputGroupAddon>
        <InputGroupInput aria-label="Search operators and stages" placeholder="Search operators, stages, materials…" />
        <InputGroupAddon align="inline-end">
            <Kbd>⌘K</Kbd>
        </InputGroupAddon>
    </InputGroup>
);

export const WithUnits = () => (
    <InputGroup className="max-w-sm">
        <InputGroupAddon>
            <Zap aria-hidden="true" />
            <InputGroupText>Budget</InputGroupText>
        </InputGroupAddon>
        <InputGroupInput aria-label="Sanity budget" defaultValue="240" />
        <InputGroupAddon align="inline-end">
            <InputGroupText>sanity / day</InputGroupText>
        </InputGroupAddon>
    </InputGroup>
);

export const WithTextarea = () => (
    <InputGroup className="max-w-sm">
        <InputGroupTextarea aria-label="Operator note" defaultValue="Mlynar wants S3M3 before E2 60 — the Arts damage carries every Chapter 8 boss run." />
        <InputGroupAddon align="block-end">
            <Badge variant="secondary">Draft</Badge>
            <Button className="ml-auto" size="sm">
                <Send aria-hidden="true" />
                Save note
            </Button>
        </InputGroupAddon>
    </InputGroup>
);

export const States = () => (
    <div className="flex max-w-sm flex-col gap-3">
        <InputGroup>
            <InputGroupAddon>
                <Search aria-hidden="true" />
            </InputGroupAddon>
            <InputGroupInput aria-label="Search stages" placeholder="Search stages…" />
        </InputGroup>
        <InputGroup>
            <InputGroupAddon>
                <Search aria-hidden="true" />
            </InputGroupAddon>
            <InputGroupInput aria-invalid aria-label="Doctor UID" defaultValue="not-a-uid" />
            <InputGroupAddon align="inline-end">
                <InputGroupText className="text-destructive">Invalid UID</InputGroupText>
            </InputGroupAddon>
        </InputGroup>
        <InputGroup>
            <InputGroupAddon>
                <Search aria-hidden="true" />
            </InputGroupAddon>
            <InputGroupInput aria-label="Search enemies" disabled placeholder="Sync your account to search enemies" />
        </InputGroup>
    </div>
);
