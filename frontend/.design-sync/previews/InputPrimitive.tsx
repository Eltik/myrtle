import { Input, InputGroup, InputGroupAddon, InputGroupText, InputPrimitive, Label } from "frontend";
import { Search } from "lucide-react";

// `Input` is the styled wrapper: it renders `InputPrimitive` (Base UI's Input) inside a
// bordered span that owns the ring/shadow tokens. These are the two ways it shows up.
const CONTROL = "h-8.5 w-full min-w-0 rounded-[inherit] px-[calc(--spacing(3)-1px)] leading-8.5 outline-none placeholder:text-muted-foreground/72 sm:h-7.5 sm:leading-7.5";

export const ThroughInput = () => (
    <div className="flex max-w-sm flex-col gap-2">
        <Label htmlFor="preview-primitive-name">Tier list name</Label>
        <Input defaultValue="Endgame DPS rankings" id="preview-primitive-name" />
        <span className="text-muted-foreground text-xs">Shown on the tier list page and anywhere the list is shared.</span>
    </div>
);

export const InsideInputGroup = () => (
    <InputGroup className="max-w-sm">
        <InputGroupAddon>
            <Search aria-hidden="true" className="size-4" />
            <InputGroupText>Stage</InputGroupText>
        </InputGroupAddon>
        <InputPrimitive aria-label="Stage code" className={CONTROL} defaultValue="S4-1" placeholder="e.g. CE-6" />
    </InputGroup>
);

export const States = () => (
    <div className="flex max-w-sm flex-col gap-3">
        <Input defaultValue="Dr. Kal'tsit#4417" />
        <Input aria-invalid defaultValue="10023481x" />
        <Input disabled placeholder="Sync your account to edit" />
    </div>
);
