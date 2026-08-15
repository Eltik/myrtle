import { Button, Popover, PopoverClose, PopoverDescription, PopoverPopup, PopoverTitle, PopoverTrigger } from "frontend";

export const ResetFilters = () => (
    <div className="flex min-h-70 w-full items-start justify-center pt-4">
        <Popover open>
            <PopoverTrigger render={<Button size="sm" variant="outline" />}>Reset filters</PopoverTrigger>
            <PopoverPopup align="center" className="w-72" sideOffset={8}>
                <div className="flex flex-col gap-3">
                    <PopoverTitle>Clear 6 filters?</PopoverTitle>
                    <PopoverDescription>Rarity, class, module and faction filters will all be reset. Your sort order is kept.</PopoverDescription>
                    <div className="flex justify-end gap-2">
                        <PopoverClose render={<Button size="sm" variant="ghost" />}>Keep them</PopoverClose>
                        <Button size="sm">Reset</Button>
                    </div>
                </div>
            </PopoverPopup>
        </Popover>
    </div>
);

export const CompactTitle = () => (
    <div className="flex min-h-70 w-full items-start justify-center pt-4">
        <Popover open>
            <PopoverTrigger render={<Button size="sm" variant="outline" />}>Sanity cost</PopoverTrigger>
            <PopoverPopup align="start" className="w-64" sideOffset={8}>
                <div className="flex flex-col gap-2">
                    <PopoverTitle className="text-sm">Farming 1-7</PopoverTitle>
                    <PopoverDescription>21 runs at 6 sanity each to finish the current Orirock Cube plan.</PopoverDescription>
                    <div className="flex items-center justify-between gap-4 font-sans text-sm">
                        <span className="text-muted-foreground">Total sanity</span>
                        <span className="font-mono text-foreground tabular-nums">126</span>
                    </div>
                </div>
            </PopoverPopup>
        </Popover>
    </div>
);
