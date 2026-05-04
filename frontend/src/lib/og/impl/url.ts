import { OG_CONFIG } from "./config";
import { type OgKind, ogRegistry } from "./registry";

export function ogURL<K extends OgKind>(kind: K, id: string, data: Parameters<(typeof ogRegistry)[K]["template"]>[0]): string {
    const handler = ogRegistry[kind];
    // biome-ignore lint/suspicious/noExplicitAny: bridging typed registry to runtime
    const hash = handler.hash(data as any);
    return `${OG_CONFIG.siteURL}/api/og/${kind}/${encodeURIComponent(id)}?v=${hash}`;
}
