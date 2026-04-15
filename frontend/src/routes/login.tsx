import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/login")({
    validateSearch: (s) => ({ redirect: (s.redirect as string) ?? "/" }),
    component: RouteComponent,
});

function RouteComponent() {
    return <div>Hello "/login"!</div>;
}
