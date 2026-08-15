import { Menu, MenuItem, MenuPopup, MenuSeparator, MenuShortcut, MenuTrigger } from "frontend";
import { ChevronDownIcon, CopyIcon, PencilIcon, PlusIcon, SearchIcon, SquarePenIcon, TrashIcon, DownloadIcon } from "lucide-react";

export const EditorActions = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Tier list editor</span>
            <Menu modal={false} open>
                <MenuTrigger className="inline-flex h-8 cursor-default items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none">
                    <span className="flex min-w-0 items-center gap-2">
                        <SquarePenIcon className="h-3.5 w-3.5 opacity-70" />
                        <span className="truncate">Edit</span>
                    </span>
                    <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" />
                </MenuTrigger>
                <MenuPopup align="start" className="w-64" sideOffset={6}>
                    <MenuItem>
                        <PlusIcon />
                        Add operator
                        <MenuShortcut>⌘K</MenuShortcut>
                    </MenuItem>
                    <MenuItem>
                        <PencilIcon />
                        Rename tier
                        <MenuShortcut>F2</MenuShortcut>
                    </MenuItem>
                    <MenuItem>
                        <CopyIcon />
                        Duplicate tier
                        <MenuShortcut>⌘D</MenuShortcut>
                    </MenuItem>
                    <MenuSeparator />
                    <MenuItem variant="destructive">
                        <TrashIcon />
                        Clear tier
                        <MenuShortcut>⌫</MenuShortcut>
                    </MenuItem>
                </MenuPopup>
            </Menu>
        </div>
    </div>
);

export const NavigationShortcuts = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Quick jump</span>
            <Menu modal={false} open>
                <MenuTrigger className="inline-flex h-8 cursor-default items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none">
                    <span className="flex min-w-0 items-center gap-2">
                        <SearchIcon className="h-3.5 w-3.5 opacity-70" />
                        <span className="truncate">Go to</span>
                    </span>
                    <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" />
                </MenuTrigger>
                <MenuPopup align="start" className="w-64" sideOffset={6}>
                    <MenuItem>
                        Operator index
                        <MenuShortcut>⌘1</MenuShortcut>
                    </MenuItem>
                    <MenuItem>
                        Stage browser
                        <MenuShortcut>⌘2</MenuShortcut>
                    </MenuItem>
                    <MenuItem>
                        Recruitment calculator
                        <MenuShortcut>⌘3</MenuShortcut>
                    </MenuItem>
                    <MenuItem>
                        <DownloadIcon />
                        Export roster
                        <MenuShortcut>⇧⌘E</MenuShortcut>
                    </MenuItem>
                </MenuPopup>
            </Menu>
        </div>
    </div>
);
