import { createFileRoute, redirect } from "@tanstack/react-router";

/** `/stories/sprites` is the library's Characters tab, where the gallery lives. */
export const Route = createFileRoute("/stories_/sprites")({
    beforeLoad: () => {
        throw redirect({ to: "/stories", search: { tab: "characters" }, statusCode: 301 });
    },
});
