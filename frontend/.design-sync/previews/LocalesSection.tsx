import { LocalesSection } from "frontend";

// Admin > Translations > Locales: the locale table (code, native name,
// direction, enabled, completion) with Add locale and per-row edit for a super
// admin; read-only for anyone else. It loads locales and translation progress
// through server functions, stubbed in the design bundle to reject; the section
// has no error branch, so a failed load reads as an empty table. Signed out
// (`authed` false) the queries never run and it holds its skeleton.

/** A super admin: Add locale in the header, the empty table. */
export const SuperAdmin = () => (
    <div className="w-full max-w-3xl p-6">
        <LocalesSection role="super_admin" authed />
    </div>
);

/** Any other admin role: the read-only note, no Add locale. */
export const ReadOnly = () => (
    <div className="w-full max-w-3xl p-6">
        <LocalesSection role="translator" authed />
    </div>
);

/** Before the session resolves: the table's skeleton. */
export const Loading = () => (
    <div className="w-full max-w-3xl p-6">
        <LocalesSection role={null} authed={false} />
    </div>
);
