import { PrivacyPage } from "frontend";

// The full `/privacy` route: ambient hero, TL;DR alert, three principle tiles,
// then the clause run (Yostar OAuth, what we collect, retention, your controls,
// contact) closed by the related-documents footer. All copy and dates are baked
// into the component, and it takes no props — so there is one story: the page.

export const Default = () => <PrivacyPage />;
