import { Button, Separator, ThemeToggle, WhatsNewButton } from "frontend";
import { Heart, Settings } from "lucide-react";
import type { ReactNode } from "react";

// The header bell. It takes no props: the unread dot is derived from a
// `@tanstack/store` singleton that reads `localStorage["myrtle-changelog-seen"]`
// once per page load and compares it with the newest announced release note.
// No key (a visitor who has never opened the dialog) counts as UNREAD. The
// click that opens the dialog is what writes the key and clears the dot.
//
// The stories set that key before the bell mounts. Doing it during render
// rather than in an effect matters: the bell's own effect is what reads the
// key, and a parent's render runs before any child effect.

const SEEN_KEY = "myrtle-changelog-seen";

function Seen({ id, children }: { id: string | null; children: ReactNode }) {
    try {
        if (id === null) window.localStorage.removeItem(SEEN_KEY);
        else window.localStorage.setItem(SEEN_KEY, id);
    } catch {
        // Storage blocked: the store hides the dot on its own.
    }
    return <>{children}</>;
}

// Fresh visitor: no seen marker, so the primary dot sits on the bell.
export const Unread = () => (
    <Seen id={null}>
        <WhatsNewButton />
    </Seen>
);

// Caught up: the marker is at or past the newest announced entry, no dot.
// (A far-future id so this story stays "read" when a newer note is filed.)
export const Read = () => (
    <Seen id="9999-12-31">
        <WhatsNewButton />
    </Seen>
);

// Its place in the header's right-hand cluster, between the theme toggle and
// the signed-out donate / settings actions.
export const InHeaderCluster = () => (
    <Seen id={null}>
        <div className="flex h-14 w-full items-center justify-end gap-0.5 border-border border-b bg-background px-4">
            <Separator orientation="vertical" className="mx-1 h-5" />
            <ThemeToggle />
            <WhatsNewButton />
            <Button variant="ghost" size="icon" aria-label="Donate">
                <Heart className="h-4 w-4" aria-hidden="true" />
            </Button>
            <Button variant="ghost" size="icon" aria-label="Settings">
                <Settings className="h-4 w-4" aria-hidden="true" />
            </Button>
        </div>
    </Seen>
);
