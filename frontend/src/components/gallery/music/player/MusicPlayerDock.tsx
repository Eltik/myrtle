import type React from "react";
import { TooltipProvider } from "#/components/ui/tooltip";
import { LyricsPanel } from "./LyricsPanel";
import { PlayerBar } from "./PlayerBar";
import { SettingsDialog, ShortcutsDialog } from "./PlayerDialogs";
import { QueuePanel } from "./QueuePanel";

/**
 * Everything the player draws, mounted once by the music page: the bar, the
 * lyrics and queue panels, and the settings and shortcuts dialogs. The bar
 * and the panels exist only while there is a queue; the shortcuts dialog is
 * always mounted because `?` opens it with nothing queued.
 *
 * The page also calls `useMusicPlayerLifetime()` and `setCatalog()` from
 * `./engine`; this component only draws.
 */
export function MusicPlayerDock(): React.ReactElement {
    return (
        <TooltipProvider delay={400}>
            <PlayerBar />
            <LyricsPanel />
            <QueuePanel />
            <SettingsDialog />
            <ShortcutsDialog />
        </TooltipProvider>
    );
}
