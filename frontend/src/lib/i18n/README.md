# i18n conventions

Read this before adding or changing any user-facing string.

## The shape

Three files per component that has text:

1. **`Foo.messages.ts`** — colocated, declares the English text.
2. **`Foo.tsx`** — calls `t("key")`; contains no English.
3. **`source-catalog.json`** — generated. Never hand-edit. `bun run i18n:extract`.

```ts
// Foo.messages.ts
import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "filters.empty": {
        text: "No operators match your filters.",
        description: "Empty state under the operator grid when every result is filtered out.",
    },
    "list.count": {
        text: "{count, plural, one {# operator} other {# operators}}",
        description: "Result count above the grid.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
```

```tsx
// Foo.tsx
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./Foo.messages";

export function Foo({ total }: { total: number }): React.ReactElement {
    const t: TypedT<typeof messages> = useT("operators");
    return <p>{t("list.count", { count: total })}</p>;
}
```

The `TypedT<typeof messages>` annotation is what turns a typo into a compile
error. The `import type` means the messages module is erased from the bundle —
English text and translator notes never ship to the client; they reach runtime
only through the generated catalog.

## Namespaces

One per feature area, matching `components/`. A namespace is the unit a
translator works through and the unit a route can lazily load, so keep a
string in the namespace of the screen it appears on.

| namespace | covers |
|---|---|
| `common` | `components/ui/**`, `Footer`, `NotFound`, `LocaleSwitcher` — anything reused everywhere |
| `nav` | `components/header/**`, `lib/registry/**` |
| `meta` | page titles and descriptions passed to `seo()` in `routes/**` |
| `home` | `components/home/**` |
| `operators` | `components/operators/**` |
| `user` | `components/user/**` |
| `tools` | `components/tools/**` |
| `tierLists` | `components/tier-lists/**` |
| `stages` | `components/stages/**` |
| `story` | `components/story/**` (the reader and the story library) |
| `enemies` | `components/enemies/**` |
| `gacha` | `components/gacha/**` |
| `stats` | `components/stats/**` |
| `changelog` | `components/changelog/**` |
| `skins` | `components/skins/**`, `components/export/**` |
| `settings` | `components/settings/**` |
| `legal` | `components/legal/**` |
| `admin` | `components/admin/**` |

## Rules

**English output must not change.** Every conversion is a refactor, not a
rewrite. With only `en` enabled the rendered page must be byte-identical:
same words, same punctuation, same ellipsis character, same surrounding
whitespace. JSX collapses whitespace in some positions and preserves it in
others — check the significant spaces around inline elements.

**Translate what a person reads or hears.** That includes visible text,
`aria-label`, `alt`, `title`, `placeholder`, `label`, tooltip content, toast
titles and descriptions, empty states, validation messages, button and menu
labels, and option lists in `constants.ts` / registry modules.

**Do not translate** developer-facing strings that never reach a user:
`throw new Error("useSidebar must be used within a SidebarProvider")`,
`console.*`, `data-*` attribute values, `className` strings, query keys,
route ids, analytics event names, or an English word being used as an API
value (`sort=name`).

**Do not translate Arknights game vocabulary.** Operator names, skill and
talent text, stage names, item names, profession and faction labels
(`formatProfession`, `formatArchetype`, `formatNationId`, `formatGroupId`,
`formatTeamId` in `lib/utils.ts`) come from game data, which ships its own
translation per region. Those belong to the game-data layer, not this catalog.

**Numbers, dates and relative times are not messages.** Use `useFormatters()`
from `#/lib/i18n`, which binds `Intl` to the active locale. A bare
`.toLocaleString()` silently uses the *browser's* locale, not the page's, and
that mismatch is invisible until a second locale exists.

**Interpolate, never concatenate.** `t("x", { count })` with an ICU message,
not `` `${n} operators` `` and not `t("a") + n + t("b")` — word order differs
between languages and a split sentence cannot be reordered.

**Plurals are ICU.** `{count, plural, one {# operator} other {# operators}}`.
Do not write `n === 1 ? "operator" : "operators"`; English has two forms,
Polish has four, Japanese has one. `#` renders the number, locale-formatted.

**Write a description for anything ambiguous.** A translator sees the string
and your description, not the screen. "Open" as a verb and "Open" as a status
are different words in most languages, and a 6-character label in a
fixed-width tile needs to say so.

**Placeholders are validated server-side.** A translation that drops a
declared `{token}` or invents a new one is rejected with a 422. That means the
set of placeholders in a message is part of its contract — changing them
invalidates existing translations for that key.

## Changing an existing string

Editing `text` changes its hash, which marks every existing translation of
that key stale in the admin panel — intentionally. Translators see what moved.
Renaming a *key* instead orphans the old one: it is deactivated, not deleted,
so its translations survive if the key comes back, but nothing inherits them.
Prefer editing text over renaming keys.

## Commands

```
bun run i18n:extract   regenerate the source catalog
bun run i18n:check     fail if the catalog is stale (CI)
bun run i18n:sync      push keys to the backend (needs SERVICE_KEY + BACKEND_URL)
```
