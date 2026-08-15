import { LeaderboardHero } from "frontend";

const HOURS_AGO_2 = new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString();
const MOMENTS_AGO = new Date(Date.now() - 20 * 1000).toISOString();
const DAYS_AGO_3 = new Date(Date.now() - 3 * 24 * 60 * 60 * 1000).toISOString();

export const Overview = () => <LeaderboardHero rankedDoctors={18432} topScore={9418} updatedAt={HOURS_AGO_2} />;

export const JustRefreshed = () => <LeaderboardHero rankedDoctors={18437} topScore={9421} updatedAt={MOMENTS_AGO} />;

export const StaleSnapshot = () => <LeaderboardHero rankedDoctors={17980} topScore={9377} updatedAt={DAYS_AGO_3} />;

export const Loading = () => <LeaderboardHero isLoading rankedDoctors={null} topScore={null} updatedAt={null} />;
