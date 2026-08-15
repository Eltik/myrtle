import { TierListsHero } from "frontend";

export const SignedOut = () => <TierListsHero total={182} canCreate={false} />;

export const CanPublish = () => <TierListsHero total={182} canCreate />;

export const NoListsYet = () => <TierListsHero total={0} canCreate={false} />;
