import type React from "react";
import { TooltipProvider } from "#/components/ui/tooltip";
import { useMusicSessionLifetime } from "./engine";
import { LyricsPanel } from "./LyricsPanel";
import { PlayerBar } from "./PlayerBar";
import { SettingsDialog, ShortcutsDialog } from "./PlayerDialogs";
import { QueuePanel } from "./QueuePanel";

/**
 * Everything the player draws: the bar, the lyrics and queue panels, and the
 * settings and shortcuts dialogs. The bar and the panels exist only while
 * there is a queue; the shortcuts dialog is always mounted because `?` opens
 * it with nothing queued.
 *
 * `offPage` is true on every route but the music page, where the bar's cover
 * becomes the way back to it.
 */
export function MusicPlayerDock({ offPage = false }: { offPage?: boolean }): React.ReactElement {
    return (
        <TooltipProvider delay={400}>
            <PlayerBar offPage={offPage} />
            <LyricsPanel />
            <QueuePanel />
            <SettingsDialog />
            <ShortcutsDialog />
        </TooltipProvider>
    );
}

/**
 * The one dock on the site, mounted by the root layout on the music page and,
 * once the player has sounded (`./presence`), on every other route. It holds
 * the player's site-wide lifetime, so leaving the music page keeps the music.
 * The music page adds `setCatalog()` and its keyboard shortcuts on top.
 */
export function MusicHost({ offPage }: { offPage: boolean }): React.ReactElement {
    useMusicSessionLifetime();
    return <MusicPlayerDock offPage={offPage} />;
}
