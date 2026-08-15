import { Home } from "frontend";

// The whole landing route: hero → feature strip → community tier-list gallery.
// It takes no props and owns no state beyond the command-palette handle, so the
// single story is the page itself.
//
// The tier-list gallery below the strip is data-coupled; with the stubbed server
// functions and a no-retry query client it settles into its own empty/error
// branch rather than spinning.

export const LandingPage = () => <Home />;
