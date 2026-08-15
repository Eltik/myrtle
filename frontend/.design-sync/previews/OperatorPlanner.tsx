import { OperatorPlanner } from "frontend";

// The planner tool page. Every plan is stored against the signed-in account, so
// the page is auth-gated: with no session the whole tool collapses to its
// sign-in prompt, which is what a preview can honestly show.
export const SignedOut = () => <OperatorPlanner />;
