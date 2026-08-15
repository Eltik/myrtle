import { ActiveIndicator } from "frontend";

// A single glyph with a primary-tinted glow that marks the current route in the
// mobile navigation drawer. `MobileNav` renders it as the last child of the
// active `DrawerMenuItem`, where its `ml-auto` pushes it to the trailing edge -
// so the honest preview is a drawer-style menu list with one row marked.

const ROW = "flex h-10 w-full items-center rounded-md px-3 font-sans text-sm text-foreground";
const ACTIVE = `${ROW} bg-accent text-accent-foreground`;
const IDLE = `${ROW} text-muted-foreground`;

const GroupLabel = ({ children }: { children: React.ReactNode }) => <p className="m-0 mb-1 px-3 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-widest">{children}</p>;

export const InNavDrawer = () => (
    <div className="w-72 rounded-xl border border-border bg-popover p-2">
        <GroupLabel>Navigation</GroupLabel>
        <div className="flex flex-col gap-1">
            <div className={IDLE}>Home</div>
            <div className={IDLE}>Collection</div>
            <div className={IDLE}>Tools</div>
            <div className={ACTIVE}>
                Tier Lists
                <ActiveIndicator />
            </div>
            <div className={IDLE}>Players</div>
        </div>
    </div>
);

export const OnSingleRow = () => (
    <div className="w-72 rounded-xl border border-border bg-popover p-2">
        <div className={ACTIVE}>
            Gacha
            <ActiveIndicator />
        </div>
    </div>
);

export const AcrossSections = () => (
    <div className="flex w-72 flex-col gap-3 rounded-xl border border-border bg-popover p-2">
        <div>
            <GroupLabel>Collection</GroupLabel>
            <div className="flex flex-col gap-1">
                <div className={IDLE}>Operators</div>
                <div className={IDLE}>Enemies</div>
                <div className={ACTIVE}>
                    Stages
                    <ActiveIndicator />
                </div>
            </div>
        </div>
        <div>
            <GroupLabel>Account</GroupLabel>
            <div className="flex flex-col gap-1">
                <div className={IDLE}>My Tier Lists</div>
                <div className={IDLE}>Settings</div>
            </div>
        </div>
    </div>
);
