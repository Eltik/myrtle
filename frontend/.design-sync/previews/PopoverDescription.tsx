import { Button, Popover, PopoverDescription, PopoverPopup, PopoverTitle, PopoverTrigger } from "frontend";
import { InfoIcon } from "lucide-react";

export const MethodologyNote = () => (
    <div className="flex min-h-70 w-full items-start justify-center pt-4">
        <Popover open>
            <PopoverTrigger
                render={
                    <Button aria-label="How is the tier list scored?" size="icon" variant="ghost">
                        <InfoIcon className="h-4 w-4" />
                    </Button>
                }
            />
            <PopoverPopup align="center" className="w-72" sideOffset={8}>
                <div className="flex flex-col gap-2">
                    <PopoverTitle className="text-sm">How placements are scored</PopoverTitle>
                    <PopoverDescription>Every doctor gets one vote per operator. Votes from accounts with a synced roster are weighted 1.5×, and placements older than two versions decay out.</PopoverDescription>
                </div>
            </PopoverPopup>
        </Popover>
    </div>
);

export const DescriptionOnly = () => (
    <div className="flex min-h-70 w-full items-start justify-center pt-4">
        <Popover open>
            <PopoverTrigger render={<Button size="sm" variant="outline" />}>Why is Muelsyse ranked S?</PopoverTrigger>
            <PopoverPopup align="start" className="w-72" sideOffset={8}>
                <PopoverDescription>Her S3 summons cover an entire lane on stall maps, which is worth more in CC than raw DPS. Community placement: S (312 votes, updated 4 days ago).</PopoverDescription>
            </PopoverPopup>
        </Popover>
    </div>
);
