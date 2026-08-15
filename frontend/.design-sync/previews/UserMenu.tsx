import { UserMenu } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// The signed-in chip the header renders on the right of the toolbar: an avatar +
// nickname link glued to a chevron that opens the account menu. Ported from
// `src/components/header/Header.tsx`, which passes the `useAuth()` triple straight
// through.

const noop = async () => {};

const DOCTOR = {
    id: "9f2b0c74-3c8e-4a11-9d21-7c6b0f5a4e18",
    uid: "10345678",
    nickname: "Eltik",
    nick_number: "1734",
    level: 120,
    avatar_id: null,
    secretary: "char_4064_mlynar",
    secretary_skin_id: null,
    resume_id: null,
    role: "user",
    server: "en",
    total_score: 8734,
    grade: "A",
    public_profile: true,
    store_gacha: true,
    share_stats: true,
    exp: 12045,
    orundum: 18420,
    lmd: 4218993,
    sanity: 132,
    max_sanity: 135,
    gacha_tickets: 21,
    ten_pull_tickets: 3,
    monthly_sub_end: 1718409600,
    register_ts: 1584230400,
    last_online_ts: 1715766000,
    resume: "Rhodes Island, Reserve Op A4",
    friend_num_limit: 200,
    cumulative_signin: 1284,
    operator_count: 231,
    item_count: 1873,
    skin_count: 96,
    non_default_skin_count: 41,
    updated_at: "2024-05-15T09:12:00.000Z",
};

const EDITOR = { ...DOCTOR, id: "1c40a9e8-27b1-4f6d-8f0b-2ae5c1d33a90", uid: "20114455", nickname: "Kal'tsit", nick_number: "0003", level: 128, secretary: "char_263_skadi", role: "tier_list_admin" };

// The menu owns its own open state (no `open` prop), so the chevron is clicked on
// mount. Base UI wires the trigger after the first paint - a click fired straight
// from the effect is silently dropped, so wait two frames.
const AutoOpen = ({ children }: { children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => {
                ref.current?.querySelector<HTMLButtonElement>('button[aria-label="Open user menu"]')?.click();
                // Base UI focuses the popup's first tabbable node - the nickname link -
                // and its brand-red ring reads as an error box in a still. Drop it once
                // the menu has settled.
                requestAnimationFrame(() => requestAnimationFrame(() => (document.activeElement as HTMLElement | null)?.blur()));
            });
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, []);
    return (
        <div className="flex min-h-[520px] w-full justify-center" ref={ref}>
            {children}
        </div>
    );
};

export const SignedIn = () => (
    <div className="flex w-full justify-center">
        <UserMenu user={DOCTOR} loading={false} logout={noop} />
    </div>
);

export const AccountMenuOpen = () => (
    <AutoOpen>
        <UserMenu user={DOCTOR} loading={false} logout={noop} />
    </AutoOpen>
);

export const AdminAccount = () => (
    <AutoOpen>
        <UserMenu user={EDITOR} loading={false} logout={noop} />
    </AutoOpen>
);

export const SessionLoading = () => (
    <div className="flex w-full justify-center">
        <UserMenu user={null} loading={true} logout={noop} />
    </div>
);

export const SignedOut = () => (
    <div className="flex w-full justify-center">
        <UserMenu user={null} loading={false} logout={noop} />
    </div>
);
