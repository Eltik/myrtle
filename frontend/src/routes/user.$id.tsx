import { createFileRoute } from "@tanstack/react-router";
import { UserProfile } from "#/components/user/UserProfile";
import { operatorsIndexQueryOptions, operatorsListQueryOptions } from "#/lib/api/operators";
import { userQueryOptions, userRosterQueryOptions } from "#/lib/api/user";
import { ogURL } from "#/lib/og/impl/url";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/user/$id")({
    component: RouteComponent,
    loader: async ({ context, params }) => {
        const [user] = await Promise.all([context.queryClient.ensureQueryData(userQueryOptions(params.id)), context.queryClient.prefetchQuery(userRosterQueryOptions(params.id)), context.queryClient.prefetchQuery(operatorsIndexQueryOptions()), context.queryClient.prefetchQuery(operatorsListQueryOptions())]);
        return user;
    },
    head: ({ loaderData, params }) => {
        if (!loaderData) return seo({ title: "Doctor", path: `/user/${params.id}` });
        const ogData = {
            nickname: loaderData.nickname ?? "Doctor",
            level: loaderData.level,
            grade: loaderData.grade,
            totalScore: loaderData.total_score,
        };
        return seo({
            title: ogData.nickname,
            description: loaderData.resume ? loaderData.resume : `Doctor profile${loaderData.level != null ? ` • Lv ${loaderData.level}` : ""}${loaderData.grade ? ` • ${loaderData.grade}` : ""}`,
            image: ogURL("user", params.id, ogData),
            path: `/user/${params.id}`,
            type: "profile",
        });
    },
});

function RouteComponent() {
    return <UserProfile />;
}
