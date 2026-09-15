import type { MessageValues } from "./format";

/**
 * The declaration side of the runtime: where a string's English text actually
 * lives.
 *
 * `t("pagination.previous")` at a call site says which string to render but
 * not what it says, which is useless as the source of a source catalog. Rather
 * than smuggling the English into a third argument at every call site - where
 * it would be duplicated across the several components that share a key, and
 * would have to be kept in sync by hand - the text lives once in a
 * `*.messages.ts` module colocated with the components that use it:
 *
 * ```ts
 * // pagination.messages.ts
 * export const namespace = "common";
 *
 * export const messages = {
 *     "pagination.previous": { text: "Previous", description: "Pager control." },
 * } satisfies MessageMap;
 *
 * export const { keys } = defineMessages({ namespace, messages });
 * ```
 *
 * The extractor then has two independent sources it can cross-check: these
 * modules say what text exists, and the `t()` call sites say what text is
 * used. Either one without the other is a warning - a defined key nobody
 * renders is dead weight in the translators' queue, and a rendered key nobody
 * defined would fall through to showing its own raw identifier.
 */
export interface IMessageDefinition {
    /** English source text, in the ICU subset `format.ts` implements. */
    text: string;
    /**
     * Context for whoever translates this. Worth writing whenever the English
     * is short enough to be ambiguous: "Open" the verb and "Open" the status
     * are different words in most languages.
     */
    description?: string;
}

/** One module's message table: local key -> definition. */
export type MessageMap = Record<string, IMessageDefinition>;

export interface IDefineMessagesInput<TMessages extends MessageMap> {
    /** The namespace these keys belong to, or omitted for the root. */
    namespace?: string;
    messages: TMessages;
    /**
     * Set on a module whose keys are rendered through a variable rather than a
     * literal - a registry or constants table that stores `labelKey` and lets
     * a component resolve it with `t(item.labelKey)`.
     *
     * The extractor cross-checks declared keys against literal `t("...")` call
     * sites, which is what catches a typo and a dead key. That check cannot
     * see an indirect call, so without this flag every registry key reports
     * as unused and the real dead keys get lost in the noise. This says "the
     * usage of these keys is not statically visible; do not expect to find
     * it".
     */
    dynamic?: boolean;
}

/**
 * Every declared key mapped to itself, so `keys.previous` has the literal type
 * `"previous"` rather than `string`. Useful where a key crosses a module
 * boundary and a plain literal would lose the compile-time check.
 */
export type MessageKeyMap<TMessages extends MessageMap> = {
    readonly [K in keyof TMessages & string]: K;
};

export interface IDefinedMessages<TMessages extends MessageMap> {
    /** Always a string; the root namespace is `""`, matching `useT()`. */
    namespace: string;
    messages: TMessages;
    keys: MessageKeyMap<TMessages>;
    /** See `IDefineMessagesInput.dynamic`. */
    dynamic: boolean;
}

/**
 * A `TFunction` narrowed to the keys one module declares.
 *
 * `const t: TypedT<typeof messages> = useT("common")` is assignable - a
 * function taking any `string` key satisfies one taking a narrower union - and
 * it buys autocomplete on the literal plus a compile error on a typo, while
 * leaving the call site as the plain `t("pagination.previous")` that the
 * extractor reads.
 */
export type TypedT<TMessages extends MessageMap> = (key: keyof TMessages & string, values?: MessageValues) => string;

/**
 * Declare a module's messages.
 *
 * This is deliberately almost a no-op at runtime: the value it returns is only
 * the key map and the namespace, because the text itself is never read from
 * here at render time - it is extracted into `source-catalog.json` at build
 * time and served from the database at run time. Keeping it a plain function
 * over a plain object literal is what lets the extractor read these modules
 * with the TypeScript parser instead of importing and executing them.
 */
export function defineMessages<TMessages extends MessageMap>({ namespace, messages, dynamic }: IDefineMessagesInput<TMessages>): IDefinedMessages<TMessages> {
    const keys = Object.fromEntries(Object.keys(messages).map((key) => [key, key])) as MessageKeyMap<TMessages>;
    return { namespace: namespace ?? "", messages, keys, dynamic: dynamic ?? false };
}

/**
 * The full key `useT` will look up for a local key, mirroring the prefixing
 * rule in `context.tsx`: a key that already carries the prefix is left alone,
 * so a key written out from the root works from inside any namespace.
 *
 * The extractor applies the same rule when it builds full keys; it reads these
 * modules with the TypeScript parser rather than importing them, so it carries
 * its own copy, and this is the definition that copy is written against.
 */
export function fullMessageKey(namespace: string | undefined, key: string): string {
    const prefix = namespace ? `${namespace}.` : "";
    return prefix && !key.startsWith(prefix) ? `${prefix}${key}` : key;
}
