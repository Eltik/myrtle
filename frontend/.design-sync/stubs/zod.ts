// Design-bundle stand-in for `zod` (582 KB unminified; the bundle must stay under the
// 12 MiB upload cap). In the browser this app only evaluates schemas at module load
// (`src/lib/auth/login.ts`, `src/lib/api/contract.ts`) and runs the dev-only
// `checkSample` contract check; the schemas that gate real input live in server
// functions, which the bundle stubs. So every builder call returns the same chainable
// proxy, and parsing passes its input through unchanged.
type Chain = ((...args: unknown[]) => Chain) & { [key: string]: unknown };

const passthrough = (value: unknown) => value;
const success = (value: unknown) => ({ success: true, data: value });

const chain: Chain = new Proxy(function chainFn() {} as unknown as Chain, {
    get(_target, prop) {
        if (prop === "parse" || prop === "parseAsync") return passthrough;
        if (prop === "safeParse" || prop === "safeParseAsync") return success;
        if (prop === "then" || typeof prop === "symbol") return undefined;
        return chain;
    },
    apply() {
        return chain;
    },
});

export const z = chain;
export default chain;
