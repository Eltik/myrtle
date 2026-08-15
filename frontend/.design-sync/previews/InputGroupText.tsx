import { InputGroup, InputGroupAddon, InputGroupInput, InputGroupText, InputGroupTextarea } from "frontend";
import { Link2, Tag, Zap } from "lucide-react";

export const UnitSuffix = () => (
    <InputGroup className="max-w-sm">
        <InputGroupAddon>
            <Zap aria-hidden="true" />
        </InputGroupAddon>
        <InputGroupInput aria-label="Sanity budget" defaultValue="240" />
        <InputGroupAddon align="inline-end">
            <InputGroupText>sanity / day</InputGroupText>
        </InputGroupAddon>
    </InputGroup>
);

export const UrlPrefix = () => (
    <InputGroup className="max-w-sm">
        <InputGroupAddon>
            <Link2 aria-hidden="true" />
            <InputGroupText>myrtle.moe/u/</InputGroupText>
        </InputGroupAddon>
        <InputGroupInput aria-label="Profile slug" defaultValue="dr-kaltsit" />
    </InputGroup>
);

export const CharacterCount = () => (
    <InputGroup className="max-w-sm">
        <InputGroupAddon align="block-start" className="border-b">
            <Tag aria-hidden="true" />
            <InputGroupText>Operator note · Texas the Omertosa</InputGroupText>
        </InputGroupAddon>
        <InputGroupTextarea aria-label="Operator note" defaultValue="Best used with Módulo EXE-Y. Pairs with Skadi for the Chapter 8 escort waves." />
        <InputGroupAddon align="block-end">
            <InputGroupText>96 / 500</InputGroupText>
        </InputGroupAddon>
    </InputGroup>
);
