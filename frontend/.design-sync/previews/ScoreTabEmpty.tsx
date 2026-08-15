import { ScoreTabEmpty } from "frontend";

// The Score tab renders this whenever the backend has no grade row for the
// Doctor yet — a fresh sync, or a profile that opted out of scoring.
export const NoScoreYet = () => (
    <div className="mx-auto max-w-2xl">
        <ScoreTabEmpty />
    </div>
);
