import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerMenu, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerMenuItem, DrawerMenuSeparator, DrawerPanel, DrawerPopup, DrawerSwipeArea, DrawerTitle, DrawerTrigger } from "frontend";
import { ChevronRightIcon, SlidersHorizontalIcon } from "lucide-react";
import type { ReactNode } from "react";

// DrawerSwipeArea is an invisible fixed strip along the screen edge that listens
// for the open gesture. It ships with no background of its own, so these stories
// tint it and label it to make the hit zone legible in a static card.
const PlannerBehind = ({ children, className = "" }: { children?: ReactNode; className?: string }) => (
    <div className={`min-h-[520px] space-y-4 p-6 ${className}`}>
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Today's plan</span>
            <Badge variant="secondary">132 sanity left</Badge>
        </div>
        <div className="space-y-2">
            {[
                { stage: "1-7", runs: 12, drop: "Orirock Cube" },
                { stage: "S4-1", runs: 6, drop: "Sugar Substitute" },
                { stage: "8-14", runs: 5, drop: "RMA70-12" },
                { stage: "CE-6", runs: 4, drop: "10,000 LMD" },
                { stage: "AP-5", runs: 3, drop: "Skill Summary 3" },
            ].map((r) => (
                <div key={r.stage} className="flex items-center justify-between gap-3 rounded-lg border bg-card p-3">
                    <div className="flex flex-col">
                        <span className="font-medium text-sm">{r.stage}</span>
                        <span className="text-muted-foreground text-xs">{r.drop}</span>
                    </div>
                    <span className="font-mono text-sm tabular-nums">{r.runs} runs</span>
                </div>
            ))}
        </div>
        {children}
    </div>
);

export const BottomEdgeSwipeArea = () => (
    <Drawer position="bottom">
        <PlannerBehind />
        <DrawerSwipeArea className="flex items-center justify-center bg-primary/10 font-medium text-primary text-xs ring-1 ring-primary/30">Swipe up from this edge to open the run editor</DrawerSwipeArea>
        <DrawerPopup className="mx-auto max-w-lg" showBar>
            <DrawerHeader>
                <DrawerTitle>Edit run count</DrawerTitle>
                <DrawerDescription>8-14 · 21 sanity per run.</DrawerDescription>
            </DrawerHeader>
            <DrawerFooter>
                <DrawerClose render={<Button variant="outline" />}>Cancel</DrawerClose>
                <Button>Save</Button>
            </DrawerFooter>
        </DrawerPopup>
    </Drawer>
);

export const LeftEdgeSwipeArea = () => (
    <Drawer position="left">
        <PlannerBehind className="pl-12">
            <DrawerTrigger render={<Button size="sm" variant="outline" />}>Open navigation</DrawerTrigger>
        </PlannerBehind>
        <DrawerSwipeArea className="flex items-center justify-center bg-primary/10 text-primary ring-1 ring-primary/30">
            <ChevronRightIcon className="size-4" />
        </DrawerSwipeArea>
        <DrawerPopup className="w-70 max-w-[calc(100vw-3rem)]" showCloseButton>
            <DrawerHeader>
                <DrawerTitle>myrtle.moe</DrawerTitle>
            </DrawerHeader>
            <DrawerPanel scrollFade={false}>
                <DrawerMenu>
                    <DrawerMenuGroup>
                        <DrawerMenuGroupLabel>Navigation</DrawerMenuGroupLabel>
                        <DrawerMenuItem>Operators</DrawerMenuItem>
                        <DrawerMenuItem>Stages</DrawerMenuItem>
                        <DrawerMenuItem>Planner</DrawerMenuItem>
                    </DrawerMenuGroup>
                    <DrawerMenuSeparator />
                    <DrawerMenuGroup>
                        <DrawerMenuGroupLabel>Account</DrawerMenuGroupLabel>
                        <DrawerMenuItem>Settings</DrawerMenuItem>
                    </DrawerMenuGroup>
                </DrawerMenu>
            </DrawerPanel>
        </DrawerPopup>
    </Drawer>
);

export const DisabledSwipeArea = () => (
    <Drawer position="bottom">
        <PlannerBehind>
            <DrawerTrigger render={<Button variant="outline" />}>
                <SlidersHorizontalIcon />
                Filters
            </DrawerTrigger>
        </PlannerBehind>
        <DrawerSwipeArea className="flex items-center justify-center bg-muted/40 text-muted-foreground text-xs ring-1 ring-border" disabled>
            Edge gesture off while the route recalculates — use Filters
        </DrawerSwipeArea>
        <DrawerPopup className="mx-auto max-w-lg" showBar>
            <DrawerHeader>
                <DrawerTitle>Route filters</DrawerTitle>
                <DrawerDescription>Applies to every stage in the current plan.</DrawerDescription>
            </DrawerHeader>
            <DrawerFooter>
                <DrawerClose render={<Button variant="outline" />}>Reset</DrawerClose>
                <Button>Apply</Button>
            </DrawerFooter>
        </DrawerPopup>
    </Drawer>
);
