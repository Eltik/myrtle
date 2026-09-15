import { Fragment, isValidElement, type ReactNode, useMemo } from "react";

import { useI18n } from "./context";
import { formatMessage, type MessageValues } from "./format";
import { SOURCE_CATALOG } from "./source";

/**
 * Values a rich message accepts: everything `formatMessage` takes, plus React
 * nodes for the parts of a sentence that are markup.
 */
export type RichValues = Record<string, ReactNode | MessageValues[string]>;

/**
 * Sentinels marking where a React node goes in the formatted string.
 *
 * These are Unicode Private Use Area characters, which by definition carry no
 * meaning in any language and cannot appear in real translated text - so a
 * translator cannot accidentally produce one and split their own sentence.
 */
const OPEN = "";
const CLOSE = "";
const SPLIT = /(\d+)/;

function isNode(value: unknown): value is ReactNode {
    if (isValidElement(value)) return true;
    if (Array.isArray(value)) return true;
    // Dates are `MessageValues`, not markup - ICU must format them.
    return false;
}

/**
 * Format a message whose arguments may be React elements.
 *
 * The trick is ordering: node arguments are swapped for sentinel strings
 * FIRST, so the whole message still goes through the normal ICU formatter with
 * its scalar arguments intact. Plural selection, `#`, number and date
 * formatting all behave exactly as they do in a plain message. Only afterwards
 * is the formatted string split on the sentinels and the nodes interleaved.
 *
 * So `{count, plural, one {# result for {query}} other {# results for {query}}}`
 * works with `query` as a `<mark>` element - the plural category is still
 * chosen by `Intl.PluralRules` on the real number.
 */
function formatRich(message: string, locale: string, values?: RichValues): ReactNode {
    if (!values) return formatMessage(message, locale);

    const nodes: ReactNode[] = [];
    const scalars: MessageValues = {};

    for (const [key, value] of Object.entries(values)) {
        if (isNode(value)) {
            scalars[key] = `${OPEN}${nodes.length}${CLOSE}`;
            nodes.push(value);
        } else {
            scalars[key] = value as MessageValues[string];
        }
    }

    const rendered = formatMessage(message, locale, scalars);
    if (nodes.length === 0) return rendered;

    // `String.split` with a capturing group yields [text, index, text, ...].
    const parts = rendered.split(SPLIT);
    return parts.map((part, i) =>
        i % 2 === 1 ? (
            // biome-ignore lint/suspicious/noArrayIndexKey: positions in one formatted string are stable
            <Fragment key={i}>{nodes[Number(part)]}</Fragment>
        ) : (
            part
        ),
    );
}

export type RichTFunction = (key: string, values?: RichValues) => ReactNode;

/**
 * `useT` for a sentence that contains markup.
 *
 * This exists to kill the `.before` / `.link` / `.after` pattern. Splitting one
 * sentence into three keys around an inline `<a>` looks harmless in English,
 * where the fragments happen to sit in the right order - but a translator
 * cannot move the link, and Japanese and German routinely need it somewhere
 * else in the sentence. Three fragments is three untranslatable half-sentences.
 *
 * Instead the sentence stays one message with a `{link}` argument:
 *
 * ```tsx
 * const rt = useRichT("legal");
 * rt("privacy.security.openSource", {
 *     link: <a href={REPO_URL}>{t("privacy.security.githubLink")}</a>,
 * })
 * ```
 *
 * with the message `"…our code is open source on {link}, so you can review our
 * security practices."` - one unit, reorderable, and the placeholder validator
 * on the backend enforces that `{link}` survives translation.
 */
export function useRichT(namespace?: string): RichTFunction {
    const { locale, messages } = useI18n();

    return useMemo(() => {
        const prefix = namespace ? `${namespace}.` : "";
        return (key: string, values?: RichValues) => {
            const full = prefix && !key.startsWith(prefix) ? `${prefix}${key}` : key;
            const message = messages[full] ?? SOURCE_CATALOG[full] ?? full;
            return formatRich(message, locale, values);
        };
    }, [locale, messages, namespace]);
}

/** A `RichTFunction` narrowed to the keys one messages module declares. */
export type TypedRichT<TMessages> = (key: keyof TMessages & string, values?: RichValues) => ReactNode;
