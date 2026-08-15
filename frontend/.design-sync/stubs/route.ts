// Design-sync stub for the five `#/routes/*` modules that feature containers
// import to read typed search params (`Route.useSearch()`, `Route.fullPath`).
//
// Importing a real route module drags in routeTree.gen.ts and, through it, the
// OG image pipeline (satori/resvg) and every server loader — none of which can
// exist in a browser design-system bundle. The containers that use this are
// data-coupled and never render live in a preview; they only need the module to
// evaluate so the rest of the bundle links.
//
// Wired in through `compilerOptions.paths` in .design-sync/tsconfig.ds.json.

// `useSearch` returns permissive defaults rather than `{}`: containers destructure
// typed search params straight into state, so an empty object gives
// `useState(undefined)` and the first `.trim()`/`.map()` throws before any JSX is
// produced — the card captures blank with no reported error. These defaults let a
// route container render its real empty state instead.
export const Route = {
    fullPath: "/",
    // Array-valued keys matter as much as scalars: a container doing
    // `selectedFlairs.length` on an absent key throws exactly like an absent string.
    useSearch: () =>
        ({ q: "", search: "", page: 1, sort: "recent", tab: "", view: "", server: "", filter: "", type: "all", flair: [], tags: [] }) as Record<
            string,
            unknown
        >,
    useParams: () => ({}) as Record<string, never>,
    useLoaderData: () => undefined,
    useNavigate: () => () => Promise.resolve(),
    useRouteContext: () => ({}) as Record<string, never>,
};
