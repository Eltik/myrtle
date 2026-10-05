import { createFileRoute, redirect } from "@tanstack/react-router";

/** `/stories/sprites/<base>` is that character's sheet open over the library's Characters tab (`/stories?tab=characters&sprite=<base>`). */
export const Route = createFileRoute("/stories_/sprites_/$base")({
    beforeLoad: ({ params }) => {
        throw redirect({ to: "/stories", search: { tab: "characters", sprite: params.base }, statusCode: 301 });
    },
});
