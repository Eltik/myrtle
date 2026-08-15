import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "frontend";
import { CalendarOffIcon, SwordsIcon } from "lucide-react";

export const IconTile = () => (
    <Empty>
        <EmptyHeader>
            <EmptyMedia variant="icon">
                <SwordsIcon />
            </EmptyMedia>
            <EmptyTitle>No enemies recorded</EmptyTitle>
            <EmptyDescription>Enemies populate the handbook as you clear stages. Resync the profile to refresh what has been encountered.</EmptyDescription>
        </EmptyHeader>
    </Empty>
);

export const PlainIcon = () => (
    <Empty>
        <EmptyHeader>
            <EmptyMedia>
                <CalendarOffIcon className="size-12 text-muted-foreground" />
            </EmptyMedia>
            <EmptyTitle>No birthdays this month</EmptyTitle>
            <EmptyDescription>Nobody on Rhodes Island celebrates in February. Switch months to see who is up next.</EmptyDescription>
        </EmptyHeader>
    </Empty>
);

export const OperatorArt = () => (
    <Empty>
        <EmptyHeader>
            <EmptyMedia>
                <img alt="Mlynar" className="size-16 rounded-full border border-border" src="https://api.myrtle.moe/api/avatar/char_4064_mlynar" />
            </EmptyMedia>
            <EmptyTitle>Mlynar has no skill data yet</EmptyTitle>
            <EmptyDescription>DPS formulas for this operator have not been contributed. Pick another operator, or open the calculator to add one.</EmptyDescription>
        </EmptyHeader>
    </Empty>
);
