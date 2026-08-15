import { Button, InputGroup, InputGroupAddon, InputGroupInput, InputGroupText, InputGroupTextarea, Kbd } from "frontend";
import { Filter, Search, Send, Tag, X } from "lucide-react";

export const InlineStart = () => (
    <InputGroup className="max-w-sm">
        <InputGroupAddon>
            <Search aria-hidden="true" />
        </InputGroupAddon>
        <InputGroupInput aria-label="Search this page" placeholder="Search Doctor name or UID…" />
    </InputGroup>
);

export const InlineEnd = () => (
    <div className="flex max-w-sm flex-col gap-3">
        <InputGroup>
            <InputGroupAddon>
                <Search aria-hidden="true" />
            </InputGroupAddon>
            <InputGroupInput aria-label="Search tier lists" defaultValue="endgame dps" />
            <InputGroupAddon align="inline-end">
                <Button aria-label="Clear search" size="icon-xs" variant="ghost">
                    <X aria-hidden="true" />
                </Button>
            </InputGroupAddon>
        </InputGroup>
        <InputGroup>
            <InputGroupAddon>
                <Filter aria-hidden="true" />
            </InputGroupAddon>
            <InputGroupInput aria-label="Filter operators" placeholder="Filter operators…" />
            <InputGroupAddon align="inline-end">
                <Kbd>/</Kbd>
            </InputGroupAddon>
        </InputGroup>
    </div>
);

export const BlockStart = () => (
    <InputGroup className="max-w-sm">
        <InputGroupAddon align="block-start" className="border-b">
            <Tag aria-hidden="true" />
            <InputGroupText>Operator note · Mlynar</InputGroupText>
        </InputGroupAddon>
        <InputGroupTextarea aria-label="Operator note" defaultValue="S3M3 first. The Arts damage carries every Chapter 8 boss run." />
    </InputGroup>
);

export const BlockEnd = () => (
    <InputGroup className="max-w-sm">
        <InputGroupTextarea aria-label="Tier list reasoning" defaultValue="Muelsyse sits at S because summon uptime trivialises Chapter 8 crowd waves." />
        <InputGroupAddon align="block-end" className="border-t">
            <InputGroupText>142 / 500</InputGroupText>
            <Button className="ml-auto" size="sm">
                <Send aria-hidden="true" />
                Publish
            </Button>
        </InputGroupAddon>
    </InputGroup>
);
