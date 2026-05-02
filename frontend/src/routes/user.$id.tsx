import { createFileRoute } from "@tanstack/react-router";
import { UserProfile } from "#/components/user/UserProfile";
import { userQueryOptions } from "#/lib/api/user";

export const Route = createFileRoute("/user/$id")({
    component: RouteComponent,
    loader: ({ context, params }) => context.queryClient.ensureQueryData(userQueryOptions(params.id)),
});

function RouteComponent() {
    return <UserProfile />;
}
