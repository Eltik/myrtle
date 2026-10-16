/**
 * TREVOR IN THE READER (provisional, behind `VITE_TREVOR=1`).
 *
 * A sheet over the reader, from the right above 640 px and from the bottom
 * under it (the chapter sheet's rule). It opens on Ask, a chat with Trevor
 * (`AskChat`): the panel's suggested questions wait in its empty state and the
 * composer is pinned to the bottom. The recap so far, who speaks and the terms
 * named are the three tabs beside it. The panels are public and need no
 * model; asking needs an account.
 *
 * The reader mounts this lazily on the first open and keeps it mounted, so a
 * question in flight survives closing and reopening the sheet. While it is
 * open the reader counts it as a dialog: its hotkeys stand down, so Space in
 * the question box types a space instead of turning the page.
 *
 * Built from the kit (`Sheet`, `Tabs`, `Badge`, `Button`) and its tokens only,
 * so the coming mockups can restyle it without touching the data flow.
 */
import { useQuery } from "@tanstack/react-query";
import type React from "react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { Sheet, SheetDescription, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { Spinner } from "#/components/ui/spinner";
import { Tabs, TabsList, TabsPanel, TabsTab } from "#/components/ui/tabs";
import { useAuth } from "#/hooks/use-auth";
import { useMediaQuery } from "#/hooks/use-media-query";
import { trevorPanelsQueryOptions, trevorStatusQueryOptions } from "#/lib/api/trevor";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { parsePanel, type TrevorPanel } from "#/lib/trevor/panels";
import { type IAskDeps, useTrevorAsk } from "#/lib/trevor/useTrevorAsk";
import { AskChat } from "./AskChat";
import { RecapSection, TermsSection, WhoSection } from "./PanelSections";
import type { messages } from "./trevor.messages";

type TrevorT = TypedT<typeof messages>;

export type TrevorTab = "ask" | "recap" | "who" | "terms";

export interface ITrevorSheetProps {
    storyId: string;
    open: boolean;
    onOpenChange: (open: boolean) => void;
    /** Jump the reader to a script line of THIS story (a citation into it). */
    onJumpLine?: (line: number) => void;
    /** Test seam for the ask hook's network. */
    askDeps?: IAskDeps;
}

export function TrevorSheet({ storyId, open, onOpenChange, onJumpLine, askDeps }: ITrevorSheetProps): React.ReactElement {
    const t: TrevorT = useT("story");
    const phone = useMediaQuery("max-sm");
    const { user } = useAuth();
    const signedIn = user !== null;
    // Polled while the sheet is open, so the offline notice and the panel
    // version follow the worker; the backend caches it for 15 s anyway.
    const status = useQuery({ ...trevorStatusQueryOptions(), enabled: open, refetchInterval: open ? 30_000 : false });
    // The panel waits for the status once, so its request names the active
    // version (`?v=`) and the backend can answer it immutable.
    const panels = useQuery({ ...trevorPanelsQueryOptions(storyId, status.data?.activeVersion ?? null), enabled: open && !status.isPending });
    const panel = panels.data ? parsePanel(panels.data.panels) : null;
    const asker = useTrevorAsk(storyId, askDeps);
    const [tab, setTab] = useState<TrevorTab>("ask");
    const jump = onJumpLine
        ? (line: number) => {
              onJumpLine(line);
              onOpenChange(false);
          }
        : undefined;

    const body = (
        <TrevorBody
            panel={panel}
            loading={open && (status.isPending || panels.isPending)}
            failed={panels.isError}
            onRetry={() => void panels.refetch()}
            tab={tab}
            onTab={setTab}
            ask={<AskChat storyId={storyId} panel={panel} signedIn={signedIn} status={status.data} state={asker.state} onAsk={(q, token) => void asker.ask(q, token)} onResume={asker.resume} onReset={asker.reset} onJumpLine={jump} active={open && tab === "ask"} />}
        />
    );

    return (
        <Sheet open={open} onOpenChange={onOpenChange}>
            <SheetPopup side={phone ? "bottom" : "right"} className={phone ? "h-[88dvh]" : "sm:max-w-lg"} data-trevor-sheet>
                <SheetHeader className="pe-12">
                    <SheetTitle>{t("trevor.sheet.title")}</SheetTitle>
                    <SheetDescription>{panel?.title ?? t("trevor.sheet.subtitle")}</SheetDescription>
                </SheetHeader>
                {body}
            </SheetPopup>
        </Sheet>
    );
}

export interface ITrevorBodyProps {
    panel: TrevorPanel | null;
    loading: boolean;
    failed: boolean;
    onRetry: () => void;
    tab: TrevorTab;
    onTab: (tab: TrevorTab) => void;
    /** The chat, built by the caller so it owns the ask state. */
    ask: React.ReactNode;
}

/** The tabs and their content, apart from the sheet so a test can render them inline. */
export function TrevorBody({ panel, loading, failed, onRetry, tab, onTab, ask }: ITrevorBodyProps): React.ReactElement {
    const t: TrevorT = useT("story");
    const note = (child: React.ReactNode) => <div className="flex flex-col items-center gap-3 py-8 text-center text-muted-foreground text-sm">{child}</div>;
    // A story with no panel (or a panel that failed) still has the chat: only
    // its suggestion chips need the panel.
    const missing = loading
        ? note(
              <>
                  <Spinner className="size-5" />
                  <span>{t("trevor.panel.loading")}</span>
              </>,
          )
        : failed
          ? note(
                <>
                    <span>{t("trevor.panel.error")}</span>
                    <Button variant="outline" size="sm" className="max-sm:h-11" onClick={onRetry}>
                        {t("trevor.panel.retry")}
                    </Button>
                </>,
            )
          : note(<span>{t("trevor.panel.none")}</span>);
    return (
        <Tabs value={tab} onValueChange={(v) => onTab(v as TrevorTab)} className="min-h-0 flex-1 gap-0">
            <div className="border-border border-b px-4 pb-2 sm:px-6">
                <TabsList variant="underline" aria-label={t("trevor.sheet.sections")} className="w-full justify-start overflow-x-auto">
                    <TabsTab value="ask" className="max-sm:min-h-11">
                        {t("trevor.tab.ask")}
                    </TabsTab>
                    <TabsTab value="recap" className="max-sm:min-h-11">
                        {t("trevor.tab.recap")}
                    </TabsTab>
                    <TabsTab value="who" className="max-sm:min-h-11">
                        {t("trevor.tab.who")}
                    </TabsTab>
                    <TabsTab value="terms" className="max-sm:min-h-11">
                        {t("trevor.tab.terms")}
                    </TabsTab>
                </TabsList>
            </div>
            {/* The chat scrolls itself and pins its composer, so it sits outside the sheet's scroll panel; kept mounted so the thread survives a tab switch. */}
            <TabsPanel value="ask" keepMounted className="flex min-h-0 flex-col">
                {ask}
            </TabsPanel>
            <TabsPanel value="recap" className="min-h-0">
                <SheetPanel className="px-4 pt-4 sm:px-6">{panel ? <RecapSection panel={panel} /> : missing}</SheetPanel>
            </TabsPanel>
            <TabsPanel value="who" className="min-h-0">
                <SheetPanel className="px-4 pt-4 sm:px-6">{panel ? <WhoSection panel={panel} /> : missing}</SheetPanel>
            </TabsPanel>
            <TabsPanel value="terms" className="min-h-0">
                <SheetPanel className="px-4 pt-4 sm:px-6">{panel ? <TermsSection panel={panel} /> : missing}</SheetPanel>
            </TabsPanel>
        </Tabs>
    );
}
