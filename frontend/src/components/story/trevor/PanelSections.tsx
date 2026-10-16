/**
 * The four sections of a story's panel, drawn from its schema-1 JSON with no
 * model and no request: the recap so far, who speaks, the terms named, and
 * the suggested questions. Every fact in the JSON is already gated to the
 * story's own place in the release order by the panel builder; nothing here
 * filters or reorders it.
 */
import { ChevronDownIcon } from "lucide-react";
import type React from "react";
import { useId, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Collapsible, CollapsiblePanel, CollapsibleTrigger } from "#/components/ui/collapsible";
import { authActions } from "#/lib/auth/store";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { primerOf, type TrevorPanel, type TrevorSuggestion } from "#/lib/trevor/panels";
import { cn } from "#/lib/utils";
import type { messages } from "./trevor.messages";

type TrevorT = TypedT<typeof messages>;

const HEADING = "font-semibold text-muted-foreground text-xs uppercase tracking-wide";

function Empty({ children }: { children: React.ReactNode }): React.ReactElement {
    return <p className="py-6 text-center text-muted-foreground text-sm">{children}</p>;
}

export function RecapSection({ panel }: { panel: TrevorPanel }): React.ReactElement {
    const t: TrevorT = useT("story");
    const r = panel.recap;
    const primer = primerOf(panel);
    const hidden = Math.max(0, (r?.earlierCount ?? 0) - (r?.soFar.length ?? 0));
    const summary = r?.thisStory?.summary?.trim();
    if (!r || (!primer && !r.previous && r.soFar.length === 0 && !summary)) return <Empty>{t("trevor.recap.empty")}</Empty>;
    return (
        <div className="flex flex-col gap-5" data-trevor-section="recap">
            {primer ? (
                <section className="flex flex-col gap-1.5">
                    <h3 className={HEADING}>{t("trevor.recap.beforeYouStart")}</h3>
                    <p className="text-sm leading-relaxed">{primer}</p>
                </section>
            ) : null}
            {r.buildsOn && r.buildsOn.length > 0 ? (
                <section className="flex flex-col gap-1.5">
                    <h3 className={HEADING}>{t("trevor.recap.buildsOn")}</h3>
                    <ul className="flex flex-wrap gap-1.5">
                        {r.buildsOn.map((name) => (
                            <li key={name}>
                                <Badge variant="outline">{name}</Badge>
                            </li>
                        ))}
                    </ul>
                </section>
            ) : null}
            {r.previous?.summary ? (
                <section className="flex flex-col gap-1.5">
                    <h3 className={HEADING}>{t("trevor.recap.previous")}</h3>
                    {r.previous.title ? <p className="font-medium text-sm">{r.previous.title}</p> : null}
                    <p className="text-sm leading-relaxed">{r.previous.summary}</p>
                </section>
            ) : null}
            {r.soFar.length > 0 ? (
                <section className="flex flex-col gap-2">
                    <h3 className={HEADING}>{panel.group ? t("trevor.recap.soFar", { group: panel.group }) : t("trevor.recap.soFarPlain")}</h3>
                    <ol className="flex flex-col gap-2.5">
                        {r.soFar.map((s) => (
                            <li key={s.storyId} className="flex flex-col gap-0.5">
                                {s.title ? <span className="font-medium text-sm">{s.title}</span> : null}
                                {s.synopsis ? <span className="text-muted-foreground text-sm leading-relaxed">{s.synopsis}</span> : null}
                            </li>
                        ))}
                    </ol>
                    {hidden > 0 ? <p className="text-muted-foreground text-xs">{t("trevor.recap.earlierHidden", { count: hidden })}</p> : null}
                </section>
            ) : null}
            {summary ? (
                <Collapsible>
                    <CollapsibleTrigger className="group flex min-h-11 w-full items-center justify-between gap-2 rounded-md text-start font-medium text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring sm:min-h-9">
                        {t("trevor.recap.thisStory")}
                        <ChevronDownIcon className="size-4 shrink-0 transition-transform group-data-[panel-open]:rotate-180" aria-hidden />
                    </CollapsibleTrigger>
                    <CollapsiblePanel>
                        <p className="pt-1 text-sm leading-relaxed" data-trevor-this-story>
                            {summary}
                        </p>
                    </CollapsiblePanel>
                </Collapsible>
            ) : null}
        </div>
    );
}

export function WhoSection({ panel }: { panel: TrevorPanel }): React.ReactElement {
    const t: TrevorT = useT("story");
    const chars = panel.whosHere?.characters ?? [];
    const more = panel.whosHere?.more ?? 0;
    if (chars.length === 0) return <Empty>{t("trevor.who.empty")}</Empty>;
    return (
        <div className="flex flex-col gap-3" data-trevor-section="who">
            <ul className="flex flex-col divide-y divide-border">
                {chars.map((c) => (
                    <li key={c.name} className="flex flex-col gap-1 py-2.5 first:pt-0">
                        <div className="flex flex-wrap items-center gap-2">
                            <span className="font-medium text-sm">{c.name}</span>
                            {c.debut ? <Badge variant="info">{t("trevor.who.debut")}</Badge> : null}
                        </div>
                        <p className={cn("text-sm leading-relaxed", !c.identity && "text-muted-foreground italic")}>{c.identity ?? t("trevor.who.noIdentity")}</p>
                        <p className="flex flex-wrap gap-x-2 text-muted-foreground text-xs">
                            {!c.debut && c.firstSeen?.group ? <span>{t("trevor.who.firstSeen", { group: c.firstSeen.group })}</span> : null}
                            {c.seenInGroups && c.seenInGroups > 0 ? <span>{t("trevor.who.seenIn", { count: c.seenInGroups })}</span> : null}
                        </p>
                    </li>
                ))}
            </ul>
            {more > 0 ? <p className="text-muted-foreground text-xs">{t("trevor.who.more", { count: more })}</p> : null}
        </div>
    );
}

const TERM_KINDS = ["place", "nation", "organization", "event", "concept", "race"] as const;
type TermKind = (typeof TERM_KINDS)[number];
function isTermKind(k: string | null | undefined): k is TermKind {
    return (TERM_KINDS as readonly string[]).includes(k ?? "");
}

export function TermsSection({ panel }: { panel: TrevorPanel }): React.ReactElement {
    const t: TrevorT = useT("story");
    const items = panel.terms?.items ?? [];
    const more = panel.terms?.more ?? 0;
    const kindLabel = (k: TermKind): string => {
        switch (k) {
            case "place":
                return t("trevor.terms.kind.place");
            case "nation":
                return t("trevor.terms.kind.nation");
            case "organization":
                return t("trevor.terms.kind.organization");
            case "event":
                return t("trevor.terms.kind.event");
            case "concept":
                return t("trevor.terms.kind.concept");
            case "race":
                return t("trevor.terms.kind.race");
        }
    };
    if (items.length === 0) return <Empty>{t("trevor.terms.empty")}</Empty>;
    return (
        <div className="flex flex-col gap-3" data-trevor-section="terms">
            <dl className="flex flex-col divide-y divide-border">
                {items.map((term) => (
                    <div key={term.term} className="flex flex-col gap-1 py-2.5 first:pt-0">
                        <dt className="flex flex-wrap items-center gap-2">
                            <span className="font-medium text-sm">{term.term}</span>
                            {isTermKind(term.kind) ? <Badge variant="secondary">{kindLabel(term.kind)}</Badge> : null}
                        </dt>
                        <dd className={cn("text-sm leading-relaxed", !term.definition && "text-muted-foreground italic")}>{term.definition ?? t("trevor.terms.noDefinition")}</dd>
                        {term.firstNamed?.group ? <dd className="text-muted-foreground text-xs">{t("trevor.terms.firstNamed", { group: term.firstNamed.group })}</dd> : null}
                    </div>
                ))}
            </dl>
            {more > 0 ? <p className="text-muted-foreground text-xs">{t("trevor.terms.more", { count: more })}</p> : null}
        </div>
    );
}

/**
 * The suggested questions. One with a prepared answer opens it in place, no
 * request made; one without asks Trevor (or, signed out, offers sign-in).
 */
export function SuggestionsSection({ panel, signedIn, onAsk }: { panel: TrevorPanel; signedIn: boolean; onAsk: (question: string) => void }): React.ReactElement {
    const t: TrevorT = useT("story");
    const items = panel.suggestions?.items ?? [];
    return (
        <section className="flex flex-col gap-2" data-trevor-section="suggestions">
            <h3 className={HEADING}>{t("trevor.suggest.heading")}</h3>
            {items.length === 0 ? (
                <p className="text-muted-foreground text-sm">{t("trevor.suggest.empty")}</p>
            ) : (
                <ul className="flex flex-col gap-1.5">
                    {items.map((s) => (
                        <Suggestion key={s.id} s={s} signedIn={signedIn} onAsk={onAsk} />
                    ))}
                </ul>
            )}
        </section>
    );
}

function Suggestion({ s, signedIn, onAsk }: { s: TrevorSuggestion; signedIn: boolean; onAsk: (question: string) => void }): React.ReactElement {
    const t: TrevorT = useT("story");
    const [open, setOpen] = useState(false);
    const panelId = useId();
    const answer = s.answer?.trim();
    const onClick = () => {
        if (answer) setOpen((v) => !v);
        else if (signedIn) onAsk(s.question);
        else authActions.openLoginDialog();
    };
    return (
        <li className="rounded-md border border-border" data-trevor-suggestion={s.id}>
            <button
                type="button"
                onClick={onClick}
                aria-expanded={answer ? open : undefined}
                aria-controls={answer ? panelId : undefined}
                className="flex min-h-11 w-full items-start justify-between gap-2 rounded-md px-3 py-2 text-start text-sm transition-colors hover:bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring sm:min-h-9"
            >
                <span className="flex flex-col gap-1">
                    <span>{s.question}</span>
                    {s.fromThisStory ? (
                        <Badge variant="outline" className="self-start">
                            {t("trevor.suggest.fromThisStory")}
                        </Badge>
                    ) : null}
                </span>
                {answer ? <ChevronDownIcon className={cn("mt-0.5 size-4 shrink-0 text-muted-foreground transition-transform", open && "rotate-180")} aria-hidden /> : null}
            </button>
            {answer ? (
                <div id={panelId} hidden={!open} className="flex flex-col gap-1.5 border-border border-t px-3 py-2.5" data-trevor-bank-answer>
                    <p className="text-sm leading-relaxed">{answer}</p>
                    <p className="text-muted-foreground text-xs">{t("trevor.suggest.prepared")}</p>
                </div>
            ) : null}
        </li>
    );
}
