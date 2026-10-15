import { createFileRoute, redirect } from "@tanstack/react-router";
import Translations from "#/components/admin/sections/Translations/Translations";
import { roleMayReachSection } from "#/components/admin/shell/model";
import { parseTranslationsSearch } from "#/components/admin/shell/search";

export const Route = createFileRoute("/_authed/admin/translations")({
    validateSearch: parseTranslationsSearch,
    beforeLoad: ({ context }) => {
        if (!roleMayReachSection("translations", context.user?.role)) throw redirect({ to: "/admin" });
    },
    component: AdminTranslationsRoute,
});

function AdminTranslationsRoute(): React.ReactElement {
    return <Translations search={Route.useSearch()} />;
}
