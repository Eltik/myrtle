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
 * Unicode Private Use Area characters: they never appear in real translated
 * text, so a translator cannot produce one and split their own sentence.
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
 * Node arguments are swapped for sentinel strings FIRST, so the message still
 * goes through the normal ICU formatter with its scalars intact (plural, `#`,
 * number and date formatting unchanged). Only then is the result split on the
 * sentinels and the nodes interleaved.
 *
 * So `{count, plural, one {# result for {query}} other {# results for {query}}}`
 * works with `query` as a `<mark>` element: the plural category is still
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
 * Replaces the `.before` / `.link` / `.after` pattern. Three keys around an
 * inline `<a>` only work where English word order happens to fit; a translator
 * cannot move the link, and Japanese and German routinely need it elsewhere.
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
 * security practices."`: one unit, reorderable, and the backend's placeholder
 * validator enforces that `{link}` survives translation.
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
