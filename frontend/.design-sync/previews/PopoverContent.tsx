import { Button, Popover, PopoverContent, PopoverDescription, PopoverTitle, PopoverTrigger } from "frontend";
import { DownloadIcon } from "lucide-react";

export const ChibiDownload = () => (
    <div className="flex min-h-70 w-full items-start justify-center pt-4">
        <Popover open>
            <PopoverTrigger
                render={
                    <Button size="sm" variant="outline">
                        <DownloadIcon className="h-3.5 w-3.5" />
                        Download
                    </Button>
                }
            />
            <PopoverContent align="start" className="w-64" sideOffset={8}>
                <div className="flex flex-col gap-3">
                    <PopoverTitle className="text-sm">Export chibi</PopoverTitle>
                    <PopoverDescription>Mlynar · Front · Idle animation</PopoverDescription>
                    <div className="flex flex-col gap-1.5">
                        <Button className="justify-start" size="sm" variant="ghost">
                            Animated GIF (512px)
                        </Button>
                        <Button className="justify-start" size="sm" variant="ghost">
                            WebM (transparent)
                        </Button>
                        <Button className="justify-start" size="sm" variant="ghost">
                            PNG sprite sheet
                        </Button>
                    </div>
                </div>
            </PopoverContent>
        </Popover>
    </div>
);

export const OperatorSummary = () => (
    <div className="flex min-h-70 w-full items-start justify-center pt-4">
        <Popover open>
            <PopoverTrigger render={<Button size="sm" variant="outline" />}>Skadi the Corrupting Heart</PopoverTrigger>
            <PopoverContent align="center" className="w-72" sideOffset={8}>
                <div className="flex flex-col gap-3">
                    <div className="flex items-center gap-3">
                        <img alt="Skadi the Corrupting Heart" className="size-10 rounded-full border border-border" src="https://api.myrtle.moe/api/avatar/char_1012_skadi2" />
                        <div className="flex flex-col">
                            <span className="font-medium font-sans text-foreground text-sm">Skadi the Corrupting Heart</span>
                            <span className="font-sans text-muted-foreground text-xs">6★ Supporter · Abyssal Hunter</span>
                        </div>
                    </div>
                    <div className="grid grid-cols-3 gap-2 font-sans text-xs">
                        <div className="flex flex-col">
                            <span className="text-muted-foreground">ATK</span>
                            <span className="font-mono text-foreground tabular-nums">1,088</span>
                        </div>
                        <div className="flex flex-col">
                            <span className="text-muted-foreground">DEF</span>
                            <span className="font-mono text-foreground tabular-nums">406</span>
                        </div>
                        <div className="flex flex-col">
                            <span className="text-muted-foreground">Trust</span>
                            <span className="font-mono text-foreground tabular-nums">200%</span>
                        </div>
                    </div>
                </div>
            </PopoverContent>
        </Popover>
    </div>
);
