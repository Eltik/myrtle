import { Button, Popover, PopoverClose, PopoverDescription, PopoverPopup, PopoverTitle, PopoverTrigger } from "frontend";
import { XIcon } from "lucide-react";

export const FooterActions = () => (
    <div className="flex min-h-70 w-full items-start justify-center pt-4">
        <Popover open>
            <PopoverTrigger render={<Button size="sm" variant="outline" />}>Remove from list</PopoverTrigger>
            <PopoverPopup align="center" className="w-72" sideOffset={8}>
                <div className="flex flex-col gap-3">
                    <PopoverTitle className="text-sm">Remove Texas the Omertosa?</PopoverTitle>
                    <PopoverDescription>She will be dropped from tier S. The placement note is kept for 30 days.</PopoverDescription>
                    <div className="flex justify-end gap-2">
                        <PopoverClose render={<Button size="sm" variant="ghost" />}>Cancel</PopoverClose>
                        <Button size="sm" variant="destructive">
                            Remove
                        </Button>
                    </div>
                </div>
            </PopoverPopup>
        </Popover>
    </div>
);

export const DismissIcon = () => (
    <div className="flex min-h-70 w-full items-start justify-center pt-4">
        <Popover open>
            <PopoverTrigger render={<Button size="sm" variant="outline" />}>Roster sync</PopoverTrigger>
            <PopoverPopup align="start" className="w-72" sideOffset={8}>
                <div className="flex flex-col gap-2">
                    <div className="flex items-start justify-between gap-2">
                        <PopoverTitle className="text-sm">Sync finished</PopoverTitle>
                        <PopoverClose
                            render={
                                <Button aria-label="Dismiss" className="size-6" size="icon" variant="ghost">
                                    <XIcon className="h-3.5 w-3.5" />
                                </Button>
                            }
                        />
                    </div>
                    <PopoverDescription>231 operators imported, 4 new since your last sync.</PopoverDescription>
                </div>
            </PopoverPopup>
        </Popover>
    </div>
);
