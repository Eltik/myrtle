import { RecruitmentCalculator } from "frontend";

// The whole recruitment tool page: tag grid and options on the left, selected
// tags plus ranked combinations on the right. The tag table and the recruitable
// operator pool both come from server functions, which are stubbed in previews —
// so the honest card is the page with an empty tag board and the results column
// showing its "pick tags to see combinations" empty state.
export const TagDataUnavailable = () => <RecruitmentCalculator />;
