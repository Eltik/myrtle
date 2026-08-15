import { Button, InputGroup, InputGroupAddon, InputGroupInput, InputGroupText, Kbd } from "frontend";
import { Hash, Search, X } from "lucide-react";

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

export const Placeholder = () => (
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

export const Numeric = () => (
    <InputGroup className="max-w-sm">
        <InputGroupAddon>
            <Hash aria-hidden="true" />
            <InputGroupText>Doctor UID</InputGroupText>
        </InputGroupAddon>
        <InputGroupInput aria-label="Doctor UID" defaultValue="10023481" inputMode="numeric" />
    </InputGroup>
);

export const DisabledAndInvalid = () => (
    <div className="flex max-w-sm flex-col gap-3">
        <InputGroup>
            <InputGroupAddon>
                <Search aria-hidden="true" />
            </InputGroupAddon>
            <InputGroupInput aria-label="Search enemies" disabled placeholder="Sync your account to search enemies" />
        </InputGroup>
        <InputGroup>
            <InputGroupAddon>
                <Hash aria-hidden="true" />
            </InputGroupAddon>
            <InputGroupInput aria-invalid aria-label="Doctor UID" defaultValue="not-a-uid" />
            <InputGroupAddon align="inline-end">
                <InputGroupText className="text-destructive">Invalid UID</InputGroupText>
            </InputGroupAddon>
        </InputGroup>
    </div>
);
