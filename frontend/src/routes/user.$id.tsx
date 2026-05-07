import { createFileRoute } from "@tanstack/react-router";
import { UserProfile } from "#/components/user/profile/UserProfile";
import { operatorsIndexQueryOptions, operatorsListQueryOptions } from "#/lib/api/operators";
import { userQueryOptions, userRosterQueryOptions } from "#/lib/api/user";
import { ogURL } from "#/lib/og/impl/url";
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

function warmOgImage(uid: string, user: IUserProfile) {
    if (typeof window !== "undefined") return;
    const url = ogURL("user", uid, buildOgData(user));
    fetch(url).catch(() => {});
}

export const Route = createFileRoute("/user/$id")({
    component: RouteComponent,
    loader: async ({ context, params }) => {
        const [user] = await Promise.all([context.queryClient.ensureQueryData(userQueryOptions(params.id)), context.queryClient.prefetchQuery(userRosterQueryOptions(params.id)), context.queryClient.prefetchQuery(operatorsIndexQueryOptions()), context.queryClient.prefetchQuery(operatorsListQueryOptions())]);
        if (user) warmOgImage(params.id, user);
        return user;
    },
    head: ({ loaderData, params }) => {
        if (!loaderData) return seo({ title: "Doctor", path: `/user/${params.id}` });
        const ogData = buildOgData(loaderData);
        const image = ogURL("user", params.id, ogData);
        const base = seo({
            title: ogData.nickname,
            description: loaderData.resume ? loaderData.resume : `Doctor profile${loaderData.level != null ? ` • Lv ${loaderData.level}` : ""}${loaderData.grade ? ` • ${loaderData.grade}` : ""}`,
            image,
            path: `/user/${params.id}`,
            type: "profile",
        });
        return {
            ...base,
            links: [...base.links, { rel: "preload", as: "image", href: image }],
        };
    },
});

function RouteComponent() {
    return <UserProfile />;
}
