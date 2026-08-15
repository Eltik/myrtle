import { Permissions } from "frontend";

// The admin panel's Tier Lists screen — browse every tier list on the platform
// and manage its View → Edit → Publish → Admin grant ladder.
//
// The catalog, the flair list and the user lookup all come from server functions
// that are stubbed in a preview, so the screen renders its empty branch: a
// zero-count list with the search field live, and the detail pane on its
// "pick a list" placeholder.
export const EmptyCatalog = () => <Permissions />;
