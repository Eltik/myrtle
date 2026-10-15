import { useQuery } from "@tanstack/react-query";
import { useSelector } from "@tanstack/react-store";
import { useEffect, useMemo } from "react";
import { useAuth } from "#/hooks/use-auth";
import { initSeen, preferredServer, seenStore, unreadOf } from "#/lib/announcements";
import { announcementsQueryOptions } from "#/lib/api/announcements";
import { useGamedataServer } from "#/lib/i18n";

/**
 * The server whose board a reader sees by default: the one their account plays on, else the one
 * their game text reads from. A server with no board (CN) falls back to the backend's default.
 */
export function useReaderServer(): string | undefined {
    const { user } = useAuth();
    const gamedataServer = useGamedataServer();
    return preferredServer(user?.server, gamedataServer);
}

/** A board, its unread notices, and the seen state behind them. `server` undefined = the reader's. */
export function useAnnouncementBoard(server?: string) {
    const reader = useReaderServer();
    const query = useQuery(announcementsQueryOptions(server ?? reader));
    const seen = useSelector(seenStore, (s) => s);

    useEffect(() => {
        initSeen();
    }, []);

    const board = query.data;
    const unread = useMemo(() => (board ? unreadOf(board.announcements, board.server, seen) : []), [board, seen]);
    return { query, board, unread, seenReady: seen.status === "ready" };
}
