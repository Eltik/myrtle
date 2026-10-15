import { AccountPanel } from "frontend";

// Settings > Account: the identity header (assistant avatar, nickname, UID,
// level, server, last sync), one re-sync control with the assistant operator,
// the data card (public profile, export) and the linked-account card (sign out,
// disconnect). Pure props: the settings page passes the signed-in profile and
// its mutation states. Synced a few hours before the capture clock (2024-05-15).
// Assistant Myrtle (`char_151_myrtle`, checked against /api/operators/index).

const noop = () => {};

const USER = {
    uid: "84230719",
    nickname: "Doctor",
    nick_number: "4127",
    level: 120,
    server: "en",
    secretary: "char_151_myrtle",
    secretary_skin_id: null,
    updated_at: "2024-05-15T06:30:00.000Z",
};

const Stage = ({ syncing = false, signingOut = false, disconnecting = false }: { syncing?: boolean; signingOut?: boolean; disconnecting?: boolean }) => (
    <div className="w-full max-w-2xl p-6">
        <AccountPanel user={USER} onResync={noop} syncing={syncing} onSignOut={noop} signingOut={signingOut} onDisconnect={noop} disconnecting={disconnecting} />
    </div>
);

/** At rest: synced this morning, every action available. */
export const Default = () => <Stage />;

/** A re-sync in flight: the button spins and reads pending. */
export const Syncing = () => <Stage syncing />;

/** A Korea-server account with no level synced yet. */
export const KoreaNoLevel = () => (
    <div className="w-full max-w-2xl p-6">
        <AccountPanel user={{ ...USER, uid: "51904473", nickname: "Rhodes", nick_number: "0311", level: null, server: "kr", secretary: "char_002_amiya" }} onResync={noop} syncing={false} onSignOut={noop} signingOut={false} onDisconnect={noop} disconnecting={false} />
    </div>
);
