import { createFileRoute } from "@tanstack/react-router";
import { UserProfile } from "#/components/user/UserProfile";
import { operatorsIndexQueryOptions, operatorsListQueryOptions } from "#/lib/api/operators";
import { userQueryOptions, userRosterQueryOptions } from "#/lib/api/user";

export const Route = createFileRoute("/user/$id")({
    component: RouteComponent,
    loader: ({ context, params }) => Promise.all([context.queryClient.ensureQueryData(userQueryOptions(params.id)), context.queryClient.prefetchQuery(userRosterQueryOptions(params.id)), context.queryClient.prefetchQuery(operatorsIndexQueryOptions()), context.queryClient.prefetchQuery(operatorsListQueryOptions())]),
});

function RouteComponent() {
    return <UserProfile />;
}
