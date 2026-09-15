import { createFileRoute } from "@tanstack/react-router";
import { UserProfile } from "#/components/user/profile/UserProfile";
import { userQueryOptions, userRosterQueryOptions } from "#/lib/api/user";
import { metaT } from "#/lib/meta";
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
        // Warm only what the header + default (Stats) tab need for SSR: the profile
        // record and the roster. Per-tab data (score, inventory, plans, enemies,
        // operators index) is fetched lazily when its tab first becomes active.
        const [user] = await Promise.all([context.queryClient.ensureQueryData(userQueryOptions(params.id)), context.queryClient.prefetchQuery(userRosterQueryOptions(params.id))]);
        if (user) warmOg("user", params.id, buildOgData(user));
        return user;
    },
    head: ({ loaderData, match, params }) => {
        const t = metaT(match.context.i18n);
        const locale = match.context.i18n?.locale;
        if (!loaderData) return seo({ title: t("user.fallbackTitle"), path: `/user/${params.id}`, locale });
        const ogData = buildOgData(loaderData);
        return seo({
            title: ogData.nickname,
            // The profile's own resume wins; the fallback is one message with
            // both optional fragments selected inside it, so a translation can
            // reorder or drop the separators rather than inherit three
            // concatenated English pieces.
            description:
                loaderData.resume ||
                t("user.description", {
                    hasLevel: loaderData.level != null ? "yes" : "no",
                    hasGrade: loaderData.grade ? "yes" : "no",
                    level: loaderData.level,
                    grade: loaderData.grade,
                }),
            image: ogURL("user", params.id, ogData),
            path: `/user/${params.id}`,
            type: "profile",
            preloadImage: true,
            locale,
        });
    },
});

function RouteComponent() {
    return <UserProfile />;
}
