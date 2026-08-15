import { MyHero } from "frontend";

const noop = () => {};

export const Default = () => <MyHero total={7} communityCount={6} communityQuota={10} officialCount={1} totalViews={48213} totalFavorites={1264} onCreate={noop} />;

export const AtQuota = () => <MyHero total={12} communityCount={10} communityQuota={10} officialCount={2} totalViews={216480} totalFavorites={9317} onCreate={noop} />;

export const NoListsYet = () => <MyHero total={0} communityCount={0} communityQuota={10} officialCount={0} totalViews={0} totalFavorites={0} onCreate={noop} />;
