/**
 * The server entry: TanStack Start's default handler, behind the anonymous
 * SSR micro-cache (`#/lib/ssr-cache`). With `SSR_CACHE=0` the wrapper hands
 * back the default handler itself, so the server is exactly the stock entry.
 */
import { createStartHandler, defaultStreamHandler } from "@tanstack/react-start/server";
import { createServerEntry } from "@tanstack/react-start/server-entry";
import { SsrMicroCache, ssrCacheConfig } from "#/lib/ssr-cache";

const cache = new SsrMicroCache(ssrCacheConfig());

export default createServerEntry({ fetch: cache.wrap(createStartHandler(defaultStreamHandler)) });
