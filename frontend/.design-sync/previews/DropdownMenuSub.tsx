import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger } from "frontend";
import { ChevronDownIcon, ClipboardListIcon, DownloadIcon, FileJsonIcon, FileTextIcon, ImageIcon, LayoutGridIcon, MoreHorizontalIcon, PencilIcon, Share2Icon, TargetIcon, TrashIcon } from "lucide-react";

export const AddToPlan = () => (
    <div className="min-h-[520px] w-full">
        <article className="flex max-w-md items-center gap-3 rounded-lg border border-border bg-card px-3 py-2.5">
            <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center overflow-hidden rounded-md border border-border bg-muted">
                <img alt="Skadi" className="h-full w-full object-cover" src="https://api.myrtle.moe/api/avatar/char_263_skadi" />
            </span>
            <div className="min-w-0 flex-1">
                <h3 className="m-0 truncate font-sans font-semibold text-[14px] text-foreground leading-tight tracking-tight">Skadi</h3>
                <p className="m-0 mt-0.5 truncate font-sans text-[11.5px] text-muted-foreground leading-snug">E2 90 · S3 M3 · Trust 200</p>
            </div>
            <span className="shrink-0 font-mono text-[10.5px] text-muted-foreground tabular-nums">added 12 May</span>
            <DropdownMenu modal={false} open>
                <DropdownMenuTrigger aria-label="List actions" className="inline-flex h-8 w-8 shrink-0 cursor-default items-center justify-center rounded-md border border-transparent text-muted-foreground">
                    <MoreHorizontalIcon className="h-4 w-4" />
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" className="min-w-52" sideOffset={6}>
                    <DropdownMenuItem>
                        <PencilIcon />
                        Edit levels &amp; skills
                    </DropdownMenuItem>
                    <DropdownMenuSub open>
                        <DropdownMenuSubTrigger>
                            <TargetIcon />
                            Add to plan
                        </DropdownMenuSubTrigger>
                        <DropdownMenuSubContent className="min-w-52">
                            <DropdownMenuItem>Elite 2 — Lv 90</DropdownMenuItem>
                            <DropdownMenuItem>Skill 3 — Mastery 3</DropdownMenuItem>
                            <DropdownMenuItem>Module X — Stage 3</DropdownMenuItem>
                            <DropdownMenuItem>Everything above</DropdownMenuItem>
                        </DropdownMenuSubContent>
                    </DropdownMenuSub>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem variant="destructive">
                        <TrashIcon />
                        Remove from roster
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </article>
    </div>
);

export const ExportAs = () => (
    <div className="min-h-[520px] w-full">
        <div className="flex max-w-md items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Roster</span>
            <DropdownMenu modal={false} open>
                <DropdownMenuTrigger className="inline-flex h-8 cursor-default items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none">
                    <span className="flex min-w-0 items-center gap-2">
                        <DownloadIcon className="h-3.5 w-3.5 opacity-70" />
                        <span className="truncate">Share &amp; export</span>
                    </span>
                    <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" />
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" className="min-w-52" sideOffset={6}>
                    <DropdownMenuItem>
                        <Share2Icon />
                        Copy public link
                    </DropdownMenuItem>
                    <DropdownMenuSub open>
                        <DropdownMenuSubTrigger>
                            <DownloadIcon />
                            Export as
                        </DropdownMenuSubTrigger>
                        <DropdownMenuSubContent className="min-w-52">
                            <DropdownMenuItem>
                                <FileJsonIcon />
                                JSON · full roster
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                                <ImageIcon />
                                PNG · shareable card
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                                <FileTextIcon />
                                Markdown table
                            </DropdownMenuItem>
                        </DropdownMenuSubContent>
                    </DropdownMenuSub>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem disabled>
                        <ClipboardListIcon />
                        Export history — Pro only
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    </div>
);
