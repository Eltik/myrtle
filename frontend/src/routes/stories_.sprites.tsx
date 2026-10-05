import { createFileRoute, redirect } from "@tanstack/react-router";

/**
 * The character gallery is the library's Characters TAB now. This path stays
 * as a redirect so a link shared while it was a page still lands on it.
 */
export const Route = createFileRoute("/stories_/sprites")({
    beforeLoad: () => {
        throw redirect({ to: "/stories", search: { tab: "characters" }, statusCode: 301 });
    },
});
