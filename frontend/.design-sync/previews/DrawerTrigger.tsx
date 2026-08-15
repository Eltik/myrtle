import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerMenu, DrawerMenuCheckboxItem, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerPanel, DrawerPopup, DrawerTitle, DrawerTrigger } from "frontend";
import { MenuIcon, SlidersHorizontalIcon } from "lucide-react";

const OperatorGrid = () => (
    <div className="grid gap-3 p-6 sm:grid-cols-3">
        {["Mlynar", "Skadi", "Muelsyse", "Texas", "Eyjafjalla", "Amiya"].map((op) => (
            <div key={op} className="rounded-lg border bg-card p-3">
                <div className="font-medium text-sm">{op}</div>
                <div className="mt-1 font-mono text-muted-foreground text-xs">E2 90 · M3/M3/M3</div>
            </div>
        ))}
    </div>
);

export const IconTriggerInHeader = () => (
    <div className="min-h-screen">
        <header className="flex items-center justify-between gap-3 border-b px-6 py-3">
            <div className="flex items-center gap-3">
                <Drawer position="left">
                    <DrawerTrigger aria-label="Open navigation menu" render={<Button size="icon" variant="ghost" />}>
                        <MenuIcon />
                    </DrawerTrigger>
                    <DrawerPopup className="w-70 max-w-[calc(100vw-3rem)]" showCloseButton>
                        <DrawerHeader>
                            <DrawerTitle>Myrtle</DrawerTitle>
                        </DrawerHeader>
                    </DrawerPopup>
                </Drawer>
                <span className="font-heading font-semibold text-lg">Operators</span>
            </div>
            <Badge variant="secondary">231 owned</Badge>
        </header>
        <OperatorGrid />
    </div>
);

export const FilterButton = () => (
    <div className="min-h-screen">
        <div className="flex flex-wrap items-center justify-between gap-3 border-b px-6 py-3">
            <div className="flex flex-wrap gap-2">
                <Badge variant="outline">6★</Badge>
                <Badge variant="outline">Guard</Badge>
                <Badge variant="outline">Owned</Badge>
            </div>
            <Drawer position="bottom">
                <DrawerTrigger render={<Button size="sm" variant="outline" />}>
                    <SlidersHorizontalIcon />
                    Filters
                    <Badge size="sm">3</Badge>
                </DrawerTrigger>
                <DrawerPopup className="mx-auto max-w-lg" showBar>
                    <DrawerHeader>
                        <DrawerTitle>Filters</DrawerTitle>
                    </DrawerHeader>
                </DrawerPopup>
            </Drawer>
        </div>
        <OperatorGrid />
    </div>
);

export const OpenedFromTrigger = () => (
    <>
        <div className="min-h-screen">
            <div className="flex flex-wrap items-center justify-between gap-3 border-b px-6 py-3">
                <span className="font-heading font-semibold text-lg">Operators</span>
                <Button size="sm" variant="outline">
                    <SlidersHorizontalIcon />
                    Filters
                </Button>
            </div>
            <OperatorGrid />
        </div>
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Filters</DrawerTitle>
                    <DrawerDescription>The trigger stays mounted behind the backdrop and regains focus on close.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Rarity</DrawerMenuGroupLabel>
                            <DrawerMenuCheckboxItem defaultChecked>6★</DrawerMenuCheckboxItem>
                            <DrawerMenuCheckboxItem>5★</DrawerMenuCheckboxItem>
                        </DrawerMenuGroup>
                    </DrawerMenu>
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Reset</DrawerClose>
                    <Button>Show 42 operators</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);
