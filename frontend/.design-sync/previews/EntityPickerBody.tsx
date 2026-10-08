import { EntityPickerBody } from "frontend";
import type { ReactNode } from "react";

// The picker's tabs, search and tiles, for a dialog of the caller's own (the
// grid editor's picker, a profile's favourites). The catalogue loads through a
// server function, stubbed in the design bundle, so the tile area renders the
// real load-failure branch with its retry.

const noop = () => {};

/** A bordered host box standing in for the caller's dialog. */
const Host = ({ children }: { children: ReactNode }) => <div style={{ height: 560 }} className="mx-auto flex w-full max-w-3xl flex-col overflow-hidden rounded-xl border border-border bg-popover pt-4">{children}</div>;

/** Three allowed types: operators first. */
export const ThreeKinds = () => (
    <div className="p-4">
        <Host>
            <EntityPickerBody kinds={["operator", "skin", "story_sprite"]} current={null} onPick={noop} />
        </Host>
    </div>
);

/** A single allowed type: one tab. */
export const OneKind = () => (
    <div className="p-4">
        <Host>
            <EntityPickerBody kinds={["enemy"]} current={null} onPick={noop} />
        </Host>
    </div>
);
