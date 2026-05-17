import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/_authed")({
    beforeLoad: ({ context, location }) => {
        if (!context.user) {
            throw redirect({ to: "/", search: { auth: "1", next: location.href } });
        }
    },
});
