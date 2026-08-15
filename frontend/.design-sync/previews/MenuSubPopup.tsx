import { Menu, MenuItem, MenuPopup, MenuSeparator, MenuSub, MenuSubPopup, MenuSubTrigger, MenuTrigger } from "frontend";
import { ChevronDownIcon, ClipboardListIcon, DownloadIcon, FileJsonIcon, FileTextIcon, ImageIcon, LayoutGridIcon, MoreHorizontalIcon, PencilIcon, Share2Icon, TrashIcon } from "lucide-react";

export const TierChoices = () => (
    <div className="min-h-[520px] w-full">
        <article className="flex max-w-md items-center gap-3 rounded-lg border border-border bg-card px-3 py-2.5">
            <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center overflow-hidden rounded-md border border-border bg-muted">
                <img alt="Mlynar" className="h-full w-full object-cover" src="https://api.myrtle.moe/api/avatar/char_4064_mlynar" />
            </span>
            <div className="min-w-0 flex-1">
                <h3 className="m-0 truncate font-sans font-semibold text-[14px] text-foreground leading-tight tracking-tight">Global Guard Rankings</h3>
                <p className="m-0 mt-0.5 truncate font-sans text-[11.5px] text-muted-foreground leading-snug">42 operators ranked · updated after Babel</p>
            </div>
            <span className="shrink-0 font-mono text-[10.5px] text-muted-foreground tabular-nums">12.4k views</span>
            <Menu modal={false} open>
                <MenuTrigger aria-label="List actions" className="inline-flex h-8 w-8 shrink-0 cursor-default items-center justify-center rounded-md border border-transparent text-muted-foreground">
                    <MoreHorizontalIcon className="h-4 w-4" />
                </MenuTrigger>
                <MenuPopup align="start" className="min-w-52" sideOffset={6}>
                    <MenuItem>
                        <PencilIcon />
                        Rename placement
                    </MenuItem>
                    <MenuSub open>
                        <MenuSubTrigger>
                            <LayoutGridIcon />
                            Move to tier
                        </MenuSubTrigger>
                        <MenuSubPopup className="min-w-52">
                            <MenuItem>S+ — Defines the meta</MenuItem>
                            <MenuItem>S — Slot on any squad</MenuItem>
                            <MenuItem>A — Strong, situational</MenuItem>
                            <MenuItem>B — Serviceable</MenuItem>
                        </MenuSubPopup>
                    </MenuSub>
                    <MenuSeparator />
                    <MenuItem variant="destructive">
                        <TrashIcon />
                        Remove from list
                    </MenuItem>
                </MenuPopup>
            </Menu>
        </article>
    </div>
);

export const ExportChoices = () => (
    <div className="min-h-[520px] w-full">
        <div className="flex max-w-md items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Tier list</span>
            <Menu modal={false} open>
                <MenuTrigger className="inline-flex h-8 cursor-default items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none">
                    <span className="flex min-w-0 items-center gap-2">
                        <DownloadIcon className="h-3.5 w-3.5 opacity-70" />
                        <span className="truncate">Share &amp; export</span>
                    </span>
                    <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" />
                </MenuTrigger>
                <MenuPopup align="start" className="min-w-52" sideOffset={6}>
                    <MenuItem>
                        <Share2Icon />
                        Copy public link
                    </MenuItem>
                    <MenuSub open>
                        <MenuSubTrigger>
                            <DownloadIcon />
                            Export as
                        </MenuSubTrigger>
                        <MenuSubPopup className="min-w-52">
                            <MenuItem>
                                <FileJsonIcon />
                                JSON · full placements
                            </MenuItem>
                            <MenuItem>
                                <ImageIcon />
                                PNG · shareable card
                            </MenuItem>
                            <MenuItem>
                                <FileTextIcon />
                                Markdown table
                            </MenuItem>
                        </MenuSubPopup>
                    </MenuSub>
                    <MenuSeparator />
                    <MenuItem disabled>
                        <ClipboardListIcon />
                        Export history — Pro only
                    </MenuItem>
                </MenuPopup>
            </Menu>
        </div>
    </div>
);
