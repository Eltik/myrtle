import { OG_CONFIG } from "./config";
import { DEFAULT_OG_PRESETS, type DefaultOgPresetSlug } from "./presets";
import { type OgKind, ogRegistry } from "./registry";

export function ogURL<K extends OgKind>(kind: K, id: string, data: Parameters<(typeof ogRegistry)[K]["template"]>[0]): string {
    const handler = ogRegistry[kind];
    // biome-ignore lint/suspicious/noExplicitAny: bridging typed registry to runtime
    const hash = handler.hash(data as any);
    return `${OG_CONFIG.siteURL}/api/og/${kind}/${encodeURIComponent(id)}?v=${hash}`;
}

export function warmOg<K extends OgKind>(kind: K, id: string, data: Parameters<(typeof ogRegistry)[K]["template"]>[0]): void {
    if (typeof window !== "undefined") return;
    fetch(ogURL(kind, id, data)).catch(() => {});
}

// Convenience for default-template OG variants registered in DEFAULT_OG_PRESETS.
// Pages call `image: defaultOgURL("gacha-community")` instead of importing the
// preset map and `ogURL` separately.
export function defaultOgURL(slug: DefaultOgPresetSlug): string {
    return ogURL("default", slug, DEFAULT_OG_PRESETS[slug]);
}
