// Design-sync stub for `#/env`.
//
// The real module is @t3-oss/env-core over `process.env` + `import.meta.env`,
// neither of which exists in a plain browser bundle — and it is imported by
// src/lib/utils.ts, where `cn()` lives, so without this stub EVERY component
// card dies with "ReferenceError: process is not defined".
//
// VITE_BACKEND_URL is the public API host from the repo's .env.example, so
// preview cards resolve real operator/item/enemy artwork instead of broken
// images. Nothing secret belongs in here — this file ships inside the bundle.
//
// Wired in through `compilerOptions.paths` in .design-sync/tsconfig.ds.json.

export const env = {
    NODE_ENV: "production" as const,
    BACKEND_URL: "https://api.myrtle.moe",
    VITE_BACKEND_URL: "https://api.myrtle.moe",
    VITE_SITE_URL: "https://myrtle.moe",
    VITE_APP_TITLE: "Myrtle",
    GITHUB_REPO: "Eltik/myrtle",
    GITHUB_TOKEN: undefined as string | undefined,
    GITHUB_BRANCH: undefined as string | undefined,
};
