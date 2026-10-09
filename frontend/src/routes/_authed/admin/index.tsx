import { createFileRoute } from "@tanstack/react-router";
import Home from "#/components/admin/sections/Home/Home";

// Every role that reaches the panel has a Home ("Inbox" for staff, "My work"
// otherwise), so there is no role redirect here.
export const Route = createFileRoute("/_authed/admin/")({
    component: AdminHomeRoute,
});

function AdminHomeRoute(): React.ReactElement {
    return <Home search={Route.useSearch()} />;
}
