import { createFileRoute } from "@tanstack/react-router";
import { UserProfile } from "#/components/user/profile/UserProfile";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { userQueryOptions, userRosterQueryOptions } from "#/lib/api/user";
import { ogURL, warmOg } from "#/lib/og/impl/url";
import { seo } from "#/lib/seo";
import type { IUserProfile } from "#/types/user";

function buildOgData(user: IUserProfile) {
    return {
        nickname: user.nickname ?? "Doctor",
        level: user.level,
        grade: user.grade,
        totalScore: user.total_score,
    };
}

export const Route = createFileRoute("/user/$id")({
    component: RouteComponent,
    loader: async ({ context, params }) => {
        const [user] = await Promise.all([context.queryClient.ensureQueryData(userQueryOptions(params.id)), context.queryClient.prefetchQuery(userRosterQueryOptions(params.id)), context.queryClient.prefetchQuery(operatorsIndexQueryOptions())]);
        if (user) warmOg("user", params.id, buildOgData(user));
        return user;
    },
    head: ({ loaderData, params }) => {
        if (!loaderData) return seo({ title: "Doctor", path: `/user/${params.id}` });
        const ogData = buildOgData(loaderData);
        return seo({
            title: ogData.nickname,
            description: loaderData.resume ? loaderData.resume : `Doctor profile${loaderData.level != null ? ` • Lv ${loaderData.level}` : ""}${loaderData.grade ? ` • ${loaderData.grade}` : ""}`,
            image: ogURL("user", params.id, ogData),
            path: `/user/${params.id}`,
            type: "profile",
            preloadImage: true,
        });
    },
});

function RouteComponent() {
    return <UserProfile />;
}
