import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/_authed/settings")({
    beforeLoad: ({ context, location }) => {
        if (!context.user) throw redirect({ to: "/login", search: { redirect: location.href } });
    },
    component: RouteComponent,
});

function RouteComponent() {
    return <div>Hello "/_authed/settings"!</div>;
}
