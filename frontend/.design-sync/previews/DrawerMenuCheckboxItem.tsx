import { Badge, Button, Drawer, DrawerClose, DrawerFooter, DrawerHeader, DrawerMenu, DrawerMenuCheckboxItem, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerMenuSeparator, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";

const FilteredListBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Operators</span>
            <Badge variant="secondary">42 of 231 shown</Badge>
        </div>
        <div className="grid gap-3 sm:grid-cols-3">
            {["Mlynar", "Thorns", "Surtr", "Blaze", "Irene", "Hoederer"].map((op) => (
                <div key={op} className="rounded-lg border bg-card p-3">
                    <div className="font-medium text-sm">{op}</div>
                    <div className="mt-1 font-mono text-muted-foreground text-xs">6★ Guard</div>
                </div>
            ))}
        </div>
    </div>
);

export const CheckmarkFilters = () => (
    <>
        <FilteredListBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Filter by class</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Class</DrawerMenuGroupLabel>
                            <DrawerMenuCheckboxItem defaultChecked>Guard</DrawerMenuCheckboxItem>
                            <DrawerMenuCheckboxItem defaultChecked>Sniper</DrawerMenuCheckboxItem>
                            <DrawerMenuCheckboxItem>Caster</DrawerMenuCheckboxItem>
                            <DrawerMenuCheckboxItem>Defender</DrawerMenuCheckboxItem>
                            <DrawerMenuCheckboxItem>Medic</DrawerMenuCheckboxItem>
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

export const SwitchVariant = () => (
    <>
        <FilteredListBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Roster display</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuCheckboxItem defaultChecked variant="switch">
                            Show unowned operators
                        </DrawerMenuCheckboxItem>
                        <DrawerMenuCheckboxItem variant="switch">Group by class</DrawerMenuCheckboxItem>
                        <DrawerMenuCheckboxItem defaultChecked variant="switch">
                            Show module stages
                        </DrawerMenuCheckboxItem>
                        <DrawerMenuCheckboxItem variant="switch">Hide E0 operators</DrawerMenuCheckboxItem>
                    </DrawerMenu>
                </DrawerPanel>
                <DrawerFooter variant="bare">
                    <DrawerClose render={<Button variant="ghost" />}>Done</DrawerClose>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const DisabledOptions = () => (
    <>
        <FilteredListBehind />
        <Drawer open position="left">
            <DrawerPopup className="w-70 max-w-[calc(100vw-3rem)]" showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Columns</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Always shown</DrawerMenuGroupLabel>
                            <DrawerMenuCheckboxItem checked disabled>
                                Name
                            </DrawerMenuCheckboxItem>
                            <DrawerMenuCheckboxItem checked disabled>
                                Rarity
                            </DrawerMenuCheckboxItem>
                        </DrawerMenuGroup>
                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Optional</DrawerMenuGroupLabel>
                            <DrawerMenuCheckboxItem defaultChecked>Trust</DrawerMenuCheckboxItem>
                            <DrawerMenuCheckboxItem>Potential</DrawerMenuCheckboxItem>
                            <DrawerMenuCheckboxItem disabled>Base skills — CN only</DrawerMenuCheckboxItem>
                        </DrawerMenuGroup>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);
