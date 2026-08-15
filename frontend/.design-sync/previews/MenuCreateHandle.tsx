import { Menu, MenuCreateHandle, MenuItem, MenuPopup, MenuSeparator, MenuTrigger } from "frontend";
import { CopyIcon, ExternalLinkIcon, MoreHorizontalIcon, PencilIcon, TrashIcon, LayoutGridIcon } from "lucide-react";

// One handle drives a single popup shared by every row in the table, so the
// menu markup is mounted once instead of once per row.
const rowMenu = MenuCreateHandle<string>();

const ROWS = [
    { id: "char_4064_mlynar", name: "Mlynar", note: "S+ · Global Guard Rankings" },
    { id: "char_263_skadi", name: "Skadi", note: "A · Global Guard Rankings" },
    { id: "char_1028_texas2", name: "Texas the Omertosa", note: "S · Global Guard Rankings" },
];

export const SharedByEveryRow = () => (
    <div className="min-h-[520px] w-full max-w-2xl">
        <div className="flex flex-col gap-2">
            {ROWS.map((op, i) => (
                <article className="flex items-center gap-3 rounded-lg border border-border bg-card px-3 py-2.5" key={op.id}>
                    <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center overflow-hidden rounded-md border border-border bg-muted">
                        <img alt={op.name} className="h-full w-full object-cover" src={"https://api.myrtle.moe/api/avatar/" + op.id} />
                    </span>
                    <div className="min-w-0 flex-1">
                        <h3 className="m-0 truncate font-sans font-semibold text-[14px] text-foreground leading-tight tracking-tight">{op.name}</h3>
                        <p className="m-0 mt-0.5 truncate font-sans text-[11.5px] text-muted-foreground leading-snug">{op.note}</p>
                    </div>
                    <MenuTrigger
                        aria-label={"Actions for " + op.name}
                        className="inline-flex h-8 w-8 shrink-0 cursor-default items-center justify-center rounded-md border border-transparent text-muted-foreground"
                        handle={rowMenu}
                        id={i === 0 ? "roster-row-menu" : undefined}
                        payload={op.id}
                    >
                        <MoreHorizontalIcon className="h-4 w-4" />
                    </MenuTrigger>
                </article>
            ))}
        </div>
        <Menu handle={rowMenu} modal={false} open triggerId="roster-row-menu">
            <MenuPopup align="end" className="min-w-44" sideOffset={6}>
                <MenuItem>
                    <ExternalLinkIcon />
                    Open operator page
                </MenuItem>
                <MenuItem>
                    <LayoutGridIcon />
                    Move to tier
                </MenuItem>
                <MenuItem>
                    <PencilIcon />
                    Edit placement note
                </MenuItem>
                <MenuItem>
                    <CopyIcon />
                    Copy share link
                </MenuItem>
                <MenuSeparator />
                <MenuItem variant="destructive">
                    <TrashIcon />
                    Remove from list
                </MenuItem>
            </MenuPopup>
        </Menu>
    </div>
);

const bulkMenu = MenuCreateHandle<string>();

export const DetachedToolbarTrigger = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">6 operators selected</span>
            <MenuTrigger
                className="inline-flex h-8 cursor-default items-center gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none"
                handle={bulkMenu}
                id="bulk-actions-trigger"
            >
                Bulk actions
            </MenuTrigger>
        </div>
        <Menu handle={bulkMenu} modal={false} open triggerId="bulk-actions-trigger">
            <MenuPopup align="end" className="min-w-52" sideOffset={6}>
                <MenuItem>
                    <LayoutGridIcon />
                    Move all to S tier
                </MenuItem>
                <MenuItem>
                    <PencilIcon />
                    Set placement note
                </MenuItem>
                <MenuItem>
                    <CopyIcon />
                    Copy as Markdown
                </MenuItem>
                <MenuSeparator />
                <MenuItem variant="destructive">
                    <TrashIcon />
                    Remove 6 operators
                </MenuItem>
            </MenuPopup>
        </Menu>
    </div>
);
