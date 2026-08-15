import { DropdownMenu, DropdownMenuContent, DropdownMenuCreateHandle, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "frontend";
import { ExternalLinkIcon, MoreHorizontalIcon, PencilIcon, TargetIcon, TrashIcon } from "lucide-react";

// A single detached menu shared by every leaderboard row: each trigger passes
// its own payload, so the popup markup is mounted once for the whole table.
const rowMenu = DropdownMenuCreateHandle<string>();

const ROSTER = [
    { id: "char_4064_mlynar", name: "Mlynar", note: "E2 90 · S3 M3 · Trust 200" },
    { id: "char_1028_texas2", name: "Texas the Omertosa", note: "E2 80 · S2 M3 · Trust 176" },
    { id: "char_180_amgoat", name: "Eyjafjalla", note: "E2 90 · S3 M3 · Trust 200" },
];

export const SharedRowMenu = () => (
    <div className="min-h-[520px] w-full max-w-2xl">
        <div className="flex flex-col gap-2">
            {ROSTER.map((op, i) => (
                <article className="flex items-center gap-3 rounded-lg border border-border bg-card px-3 py-2.5" key={op.id}>
                    <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center overflow-hidden rounded-md border border-border bg-muted">
                        <img alt={op.name} className="h-full w-full object-cover" src={"https://api.myrtle.moe/api/avatar/" + op.id} />
                    </span>
                    <div className="min-w-0 flex-1">
                        <h3 className="m-0 truncate font-sans font-semibold text-[14px] text-foreground leading-tight tracking-tight">{op.name}</h3>
                        <p className="m-0 mt-0.5 truncate font-sans text-[11.5px] text-muted-foreground leading-snug">{op.note}</p>
                    </div>
                    <DropdownMenuTrigger
                        aria-label={"Actions for " + op.name}
                        className="inline-flex h-8 w-8 shrink-0 cursor-default items-center justify-center rounded-md border border-transparent text-muted-foreground"
                        handle={rowMenu}
                        id={i === 0 ? "roster-row-menu" : undefined}
                        payload={op.id}
                    >
                        <MoreHorizontalIcon className="h-4 w-4" />
                    </DropdownMenuTrigger>
                </article>
            ))}
        </div>
        <DropdownMenu handle={rowMenu} modal={false} open triggerId="roster-row-menu">
            <DropdownMenuContent align="end" className="min-w-52" sideOffset={6}>
                <DropdownMenuItem>
                    <ExternalLinkIcon />
                    Open operator page
                </DropdownMenuItem>
                <DropdownMenuItem>
                    <TargetIcon />
                    Add to E2 plan
                </DropdownMenuItem>
                <DropdownMenuItem>
                    <PencilIcon />
                    Edit levels &amp; skills
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem variant="destructive">
                    <TrashIcon />
                    Remove from roster
                </DropdownMenuItem>
            </DropdownMenuContent>
        </DropdownMenu>
    </div>
);

const bulkMenu = DropdownMenuCreateHandle<string>();

export const BulkActions = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">12 operators selected</span>
            <DropdownMenuTrigger
                className="inline-flex h-8 cursor-default items-center gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none"
                handle={bulkMenu}
                id="bulk-actions-trigger"
            >
                Bulk actions
            </DropdownMenuTrigger>
        </div>
        <DropdownMenu handle={bulkMenu} modal={false} open triggerId="bulk-actions-trigger">
            <DropdownMenuContent align="end" className="min-w-52" sideOffset={6}>
                <DropdownMenuItem>
                    <TargetIcon />
                    Add all to E2 plan
                </DropdownMenuItem>
                <DropdownMenuItem>
                    <PencilIcon />
                    Set trust to 200
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem variant="destructive">
                    <TrashIcon />
                    Remove 12 operators
                </DropdownMenuItem>
            </DropdownMenuContent>
        </DropdownMenu>
    </div>
);
