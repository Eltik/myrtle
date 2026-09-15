import { DEFAULT_LOCALE } from "#/lib/i18n";
import type { IMetaSource } from "#/lib/meta";
import { OG_CONFIG } from "./config";
import { DEFAULT_OG_PRESETS, type DefaultOgPresetSlug, resolveDefaultOgPreset } from "./presets";
import { type OgKind, ogRegistry } from "./registry";

const MAX_WARMED_URLS = 5_000;
const warmedOgURLs = new Set<string>();

export function ogURL<K extends OgKind>(kind: K, id: string, data: Parameters<(typeof ogRegistry)[K]["template"]>[0]): string {
    const handler = ogRegistry[kind];
    // biome-ignore lint/suspicious/noExplicitAny: bridging typed registry to runtime
    const hash = handler.hash(data as any);
    return `${OG_CONFIG.siteURL}/api/og/${kind}/${encodeURIComponent(id)}?v=${hash}`;
}

export function warmOg<K extends OgKind>(kind: K, id: string, data: Parameters<(typeof ogRegistry)[K]["template"]>[0]): void {
    if (typeof window !== "undefined") return;
    const url = ogURL(kind, id, data);
    if (warmedOgURLs.has(url)) return;
    if (warmedOgURLs.size > MAX_WARMED_URLS) warmedOgURLs.clear();
    warmedOgURLs.add(url);
    fetch(url).catch(() => {});
}

// Convenience for default-template OG variants registered in DEFAULT_OG_PRESETS.
// Pages call `image: defaultOgURL("gacha-community", match.context.i18n)`
// instead of importing the preset map and `ogURL` separately.
//
// `source` is the locale + catalog a route's `head()` already has in
// `match.context.i18n`. It does two jobs: the card's text is resolved here, so
// the `?v=` content hash covers the TRANSLATED words and a translator's edit
// invalidates the image the same way a data change does; and the locale is
// carried to the handler, which has no router context and cannot infer it.
//
// The source locale is left unmarked on purpose - no `&locale=`, and the
// resolved English is identical to the literals this table used to hold - so
// every English `og:image` URL, and every PNG already cached behind it, is
// byte-for-byte what it was.
export function defaultOgURL(slug: DefaultOgPresetSlug, source?: IMetaSource | null): string {
    const url = ogURL("default", slug, resolveDefaultOgPreset(DEFAULT_OG_PRESETS[slug], source));
    const locale = source?.locale;
    return !locale || locale === DEFAULT_LOCALE ? url : `${url}&locale=${encodeURIComponent(locale)}`;
}
