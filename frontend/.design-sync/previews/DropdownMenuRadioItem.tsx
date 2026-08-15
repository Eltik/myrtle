import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuLabel, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuTrigger } from "frontend";
import { ChevronDownIcon, LanguagesIcon, TableIcon } from "lucide-react";

export const SelectedChoice = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Leaderboard</span>
            <DropdownMenu modal={false} open>
                <DropdownMenuTrigger className="inline-flex h-8 cursor-default items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none">
                    <span className="flex min-w-0 items-center gap-2">
                        <TableIcon className="h-3.5 w-3.5 opacity-70" />
                        <span className="truncate">Compact</span>
                    </span>
                    <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" />
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" className="w-64" sideOffset={6}>
                    <DropdownMenuGroup>
                        <DropdownMenuLabel>Row density</DropdownMenuLabel>
                        <DropdownMenuRadioGroup defaultValue="compact">
                            <DropdownMenuRadioItem value="compact">Compact — 32px rows</DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="cozy">Cozy — 40px rows</DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="comfortable">Comfortable — 56px rows</DropdownMenuRadioItem>
                        </DropdownMenuRadioGroup>
                    </DropdownMenuGroup>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    </div>
);

export const WithDisabledChoice = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Preferences</span>
            <DropdownMenu modal={false} open>
                <DropdownMenuTrigger className="inline-flex h-8 cursor-default items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none">
                    <span className="flex min-w-0 items-center gap-2">
                        <LanguagesIcon className="h-3.5 w-3.5 opacity-70" />
                        <span className="truncate">English</span>
                    </span>
                    <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" />
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" className="w-64" sideOffset={6}>
                    <DropdownMenuGroup>
                        <DropdownMenuLabel>Interface language</DropdownMenuLabel>
                        <DropdownMenuRadioGroup defaultValue="en">
                            <DropdownMenuRadioItem value="en">English</DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="ja">日本語</DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="ko">한국어</DropdownMenuRadioItem>
                            <DropdownMenuRadioItem disabled value="zh">
                                简体中文 — coming soon
                            </DropdownMenuRadioItem>
                        </DropdownMenuRadioGroup>
                    </DropdownMenuGroup>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    </div>
);
