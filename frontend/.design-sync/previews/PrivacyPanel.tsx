import { PrivacyPanel } from "frontend";

const noop = () => {};

export const PublicProfile = () => <PrivacyPanel settings={{ public_profile: true, store_gacha: true, share_stats: true }} onChange={noop} saving={false} />;

/** Opted out everywhere: scores still compute, nothing is exposed. */
export const FullyPrivate = () => <PrivacyPanel settings={{ public_profile: false, store_gacha: false, share_stats: false }} onChange={noop} saving={false} />;

/** Mid-save: every switch is disabled until the mutation settles. */
export const Saving = () => <PrivacyPanel settings={{ public_profile: true, store_gacha: false, share_stats: true }} onChange={noop} saving={true} />;
