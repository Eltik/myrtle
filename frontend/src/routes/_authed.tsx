import { createFileRoute, redirect } from "@tanstack/react-router";
import { signInSearch } from "#/lib/auth/search";

export const Route = createFileRoute("/_authed")({
    beforeLoad: ({ context, location }) => {
        if (!context.user) {
            throw redirect({ to: "/", search: signInSearch(location.href) });
        }
    },
});
