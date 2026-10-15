import { MyGrids } from "frontend";

// The /grids/mine route container: the signed-in user's grids as cards with
// edit and delete. The design bundle has no signed-in user, so the list query
// never starts and the page holds its real loading state: header, quota line
// and three card skeletons.
export const Loading = () => <MyGrids />;
