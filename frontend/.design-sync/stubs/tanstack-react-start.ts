// Design-sync stub for `@tanstack/react-start`.
//
// The app's `#/lib/api/*` modules define their fetchers with `createServerFn`,
// and a handful of components value-import those modules (query options, icon
// URL builders). Bundling the real package drags @tanstack/start-server-core
// into a browser bundle, which fails to resolve node:stream, node:crypto and
// the `#tanstack-start-*` virtual entries — so the whole design-system bundle
// won't build.
//
// Wired in through `compilerOptions.paths` in .design-sync/tsconfig.ds.json.
// It preserves the builder chain's SHAPE so the api modules evaluate at import
// time; only actually calling a server fn fails, which no preview does.

type ServerFn = ((...args: unknown[]) => Promise<never>) & { url: string };

interface ServerFnBuilder {
    inputValidator: (validator?: unknown) => ServerFnBuilder;
    validator: (validator?: unknown) => ServerFnBuilder;
    middleware: (middleware?: unknown) => ServerFnBuilder;
    handler: (handler?: unknown) => ServerFn;
}

export function createServerFn(_options?: unknown): ServerFnBuilder {
    const builder: ServerFnBuilder = {
        inputValidator: () => builder,
        validator: () => builder,
        middleware: () => builder,
        handler: () => {
            // Product-shaped copy on purpose: data-coupled containers render
            // `error.message` straight into their error branch, so a
            // "design-sync: ..." string would leak harness vocabulary into the
            // cards. This reads like the real offline state it stands in for.
            const fn = () => Promise.reject(new Error("Couldn't reach the server. Check your connection and try again."));
            return Object.assign(fn, { url: "" }) as ServerFn;
        },
    };
    return builder;
}

export function createMiddleware(_options?: unknown) {
    const builder = {
        server: () => builder,
        client: () => builder,
        middleware: () => builder,
    };
    return builder;
}
