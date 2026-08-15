import { Badge, Button, InputGroup, InputGroupAddon, InputGroupInput, InputGroupText, InputGroupTextarea } from "frontend";
import { Send, Tag } from "lucide-react";

export const WithFooterActions = () => (
    <InputGroup className="max-w-sm">
        <InputGroupTextarea aria-label="Tier list reasoning" defaultValue="Muelsyse sits at S because summon uptime trivialises the Chapter 8 crowd waves and she covers the Ursus escort route on her own." />
        <InputGroupAddon align="block-end" className="border-t">
            <InputGroupText>142 / 500</InputGroupText>
            <Button className="ml-auto" size="sm">
                <Send aria-hidden="true" />
                Publish
            </Button>
        </InputGroupAddon>
    </InputGroup>
);

export const WithHeaderLabel = () => (
    <InputGroup className="max-w-sm">
        <InputGroupAddon align="block-start" className="border-b">
            <Tag aria-hidden="true" />
            <InputGroupText>Operator note · Mlynar</InputGroupText>
            <Badge className="ml-auto" variant="secondary">
                Draft
            </Badge>
        </InputGroupAddon>
        <InputGroupTextarea aria-label="Operator note" defaultValue="S3M3 before E2 60. The true-damage burst carries every Chapter 8 boss run." />
    </InputGroup>
);

export const Empty = () => (
    <InputGroup className="max-w-sm">
        <InputGroupTextarea aria-label="Feedback" placeholder="Tell us what the planner got wrong…" />
        <InputGroupAddon align="block-end">
            <InputGroupText>0 / 500</InputGroupText>
        </InputGroupAddon>
    </InputGroup>
);

export const Disabled = () => (
    <InputGroup className="max-w-sm">
        <InputGroupAddon align="block-start" className="border-b">
            <InputGroupText>Official note · read only</InputGroupText>
        </InputGroupAddon>
        <InputGroupTextarea aria-label="Official note" defaultValue="Maintained by the myrtle.moe editors. Request changes in the operator thread." disabled />
    </InputGroup>
);

export const AlongsideInput = () => (
    <div className="flex max-w-sm flex-col gap-3">
        <InputGroup>
            <InputGroupAddon>
                <InputGroupText>Title</InputGroupText>
            </InputGroupAddon>
            <InputGroupInput aria-label="Note title" defaultValue="Endgame DPS rankings" />
        </InputGroup>
        <InputGroup>
            <InputGroupTextarea aria-label="Note body" defaultValue="Ranked for Chapter 8 and above. Assumes E2 60, S3M3 and the matching module." />
        </InputGroup>
    </div>
);
