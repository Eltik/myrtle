import { EnemiesList } from "frontend";

// EnemiesList is the /enemies route container: it pulls the whole enemy
// handbook and the enemy→stage index through server functions, which the design
// bundle stubs. The page chrome — breadcrumb, chip rail, search, view toggle,
// sort/page selects, export — is all real; the grid resolves to its empty state.
export const HandbookUnavailable = () => <EnemiesList />;
