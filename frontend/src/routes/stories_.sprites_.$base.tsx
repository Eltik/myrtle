import { createFileRoute, redirect } from "@tanstack/react-router";

/**
 * One character's sheet opens OVER the library's Characters tab
 * (`/stories?tab=characters&sprite=<base>`). This path stays as a redirect so
 * a sprite link shared while it was a page still opens that sheet.
 */
export const Route = createFileRoute("/stories_/sprites_/$base")({
    beforeLoad: ({ params }) => {
        throw redirect({ to: "/stories", search: { tab: "characters", sprite: params.base }, statusCode: 301 });
    },
});
