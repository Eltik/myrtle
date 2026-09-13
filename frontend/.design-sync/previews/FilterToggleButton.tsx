import { FilterToggleButton } from "frontend";
import { Search } from "lucide-react";

const noop = () => {};

// From md up (the 900px capture) this is the 36px "FILTER | funnel" button;
// below md it collapses to a 24px chevron. `data-on` (visible) lifts it onto
// the card background with a primary-tinted border; the count badge only
// shows while the panel is hidden, since the open panel lists the filters itself.

/** The panel is open: pressed look, no badge. */
export const PanelOpen = () => <FilterToggleButton visible onToggle={noop} activeCount={3} />;

/** The panel is hidden and nothing is filtered. */
export const PanelHidden = () => <FilterToggleButton visible={false} onToggle={noop} activeCount={0} />;

/** Hidden with four filters applied - the primary badge sits on the corner. */
export const HiddenWithCount = () => <FilterToggleButton visible={false} onToggle={noop} activeCount={4} />;

/** The /operators toolbar row it leads: toggle, then the search field. */
export const InToolbar = () => (
    <div className="flex w-full flex-wrap items-center gap-2.5">
        <FilterToggleButton visible={false} onToggle={noop} activeCount={2} />
        <div className="flex h-10 min-w-60 flex-1 items-center gap-2 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] px-3 sm:max-w-115 [&>svg]:shrink-0 [&>svg]:text-muted-foreground">
            <Search className="h-3.75 w-3.75" aria-hidden="true" />
            <input type="text" defaultValue="" placeholder="Search operators..." aria-label="Search operators" className="min-w-0 flex-1 appearance-none border-0 bg-transparent p-0 font-sans text-foreground text-sm leading-none outline-none placeholder:text-muted-foreground" />
        </div>
    </div>
);
