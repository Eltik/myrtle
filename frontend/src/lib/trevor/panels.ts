/**
 * THE PANEL JSON (`schema: 1`), typed for the reader.
 *
 * The backend serves a story's panel verbatim (`StoryPanels.panels` is an
 * opaque `JsonValue`), built by `trevor/scripts/panels.py` with no model. The
 * shape is the one the prototype README proposes and the 1,862 published
 * panels carry; it is not a generated binding, so it is parsed here rather
 * than cast.
 *
 * Parsing is LENIENT per item and STRICT per panel: a malformed character,
 * term or suggestion is dropped and the rest still render, while a body that
 * is not a schema-1 panel at all comes back null (the sheet then shows its
 * "nothing for this story" state rather than throwing mid-read).
 */
import { z } from "zod";

/** A list whose malformed items are dropped instead of failing the whole panel. */
function listOf<T extends z.ZodType>(item: T) {
    return z
        .array(z.unknown())
        .catch([])
        .transform((rows) =>
            rows.flatMap((row) => {
                const parsed = item.safeParse(row);
                return parsed.success ? [parsed.data as z.output<T>] : [];
            }),
        );
}

const placeRef = z.object({ storyId: z.string(), group: z.string().nullish() });

const era = z.object({ id: z.string(), name: z.string(), position: z.number() });

const thisStory = z.object({
    title: z.string().nullish(),
    synopsis: z.string().nullish(),
    summary: z.string().nullish(),
    spoiler: z.boolean().nullish(),
});

const previousStory = z.object({ storyId: z.string(), title: z.string().nullish(), summary: z.string().nullish() });

const soFarRow = z.object({ storyId: z.string(), title: z.string().nullish(), synopsis: z.string().nullish() });

/** Full text on a group's first stories; a bank link (`primer:<group>`) on the later ones. */
const beforeYouStart = z.union([z.object({ primer: z.string(), prior: z.array(z.string()).nullish() }), z.object({ bankId: z.string() })]);

const character = z.object({
    name: z.string(),
    chunks: z.number().nullish(),
    identity: z.string().nullish(),
    identitySource: z.string().nullish(),
    whoId: z.string().nullish(),
    debut: z.boolean().nullish(),
    firstSeen: placeRef.nullish(),
    seenInGroups: z.number().nullish(),
    recentGroups: z.array(z.string()).nullish(),
});

const term = z.object({
    term: z.string(),
    kind: z.string().nullish(),
    mentions: z.number().nullish(),
    definition: z.string().nullish(),
    firstNamed: placeRef.nullish(),
});

const suggestion = z.object({
    id: z.string(),
    kind: z.string(),
    question: z.string(),
    gatePos: z.number().nullish(),
    fromThisStory: z.boolean().nullish(),
    /** The bank's precomputed answer, absent when the panels were built with `--no-answers`. */
    answer: z.string().nullish(),
});

const panelSchema = z.object({
    schema: z.literal(1),
    storyId: z.string(),
    groupId: z.string().nullish(),
    group: z.string().nullish(),
    title: z.string().nullish(),
    position: z.number().nullish(),
    era: era.nullish().catch(null),
    recap: z
        .object({
            thisStory: thisStory.nullish().catch(null),
            previous: previousStory.nullish().catch(null),
            soFar: listOf(soFarRow),
            earlierCount: z.number().nullish(),
            beforeYouStart: beforeYouStart.nullish().catch(null),
            buildsOn: z.array(z.string()).nullish().catch(null),
        })
        .nullish()
        .catch(null),
    whosHere: z
        .object({ characters: listOf(character), more: z.number().nullish() })
        .nullish()
        .catch(null),
    terms: z
        .object({ items: listOf(term), more: z.number().nullish() })
        .nullish()
        .catch(null),
    suggestions: z
        .object({ items: listOf(suggestion) })
        .nullish()
        .catch(null),
});

export type TrevorPanel = z.output<typeof panelSchema>;
export type TrevorCharacter = z.output<typeof character>;
export type TrevorTerm = z.output<typeof term>;
export type TrevorSuggestion = z.output<typeof suggestion>;
export type TrevorPanelEra = z.output<typeof era>;

/** A schema-1 panel, or null for anything else. */
export function parsePanel(value: unknown): TrevorPanel | null {
    const parsed = panelSchema.safeParse(value);
    return parsed.success ? parsed.data : null;
}

/** The full primer text when this story carries one (a group's first stories), else null. */
export function primerOf(panel: TrevorPanel | null): string | null {
    const b = panel?.recap?.beforeYouStart;
    if (!b || !("primer" in b)) return null;
    const text = b.primer.trim();
    return text === "" ? null : text;
}

/** Whether a panel has anything to show in each section, in the sheet's tab order. */
export function panelCoverage(panel: TrevorPanel): { recap: boolean; who: boolean; terms: boolean; suggestions: boolean } {
    const r = panel.recap;
    return {
        recap: Boolean(r && (r.previous || r.soFar.length > 0 || primerOf(panel) || r.thisStory?.summary)),
        who: (panel.whosHere?.characters.length ?? 0) > 0,
        terms: (panel.terms?.items.length ?? 0) > 0,
        suggestions: (panel.suggestions?.items.length ?? 0) > 0,
    };
}
