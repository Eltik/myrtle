import { createFileRoute } from "@tanstack/react-router";
import { UserProfile } from "#/components/user/profile/UserProfile";
import { userQueryOptions, userRosterQueryOptions, userSupportsQueryOptions } from "#/lib/api/user";
import { metaT } from "#/lib/meta";
import { ogURL, warmOg } from "#/lib/og/impl/url";
import { buildUserOgData } from "#/lib/og/impl/user";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/user/$id")({
    component: RouteComponent,
    loader: async ({ context, params }) => {
        // Warm only what the header + default (Stats) tab need for SSR: the profile
        // record and the roster. Per-tab data (score, inventory, plans, enemies,
        // operator tables) is fetched lazily when its tab first becomes active.
        //
        // The supports are read here only for the social card: its og:image hash
        // covers every field the card draws (`buildUserOgData`), and they are one of
        // them. A failed read hashes as no supports rather than failing the page.
        const { queryClient } = context;
        const [user, supports] = await Promise.all([queryClient.ensureQueryData(userQueryOptions(params.id)), queryClient.ensureQueryData(userSupportsQueryOptions(params.id)).catch(() => null), queryClient.prefetchQuery(userRosterQueryOptions(params.id))]);
        const roster = queryClient.getQueryData(userRosterQueryOptions(params.id).queryKey) ?? null;
        const og = user ? buildUserOgData(user, supports, roster) : null;
        if (og) warmOg("user", params.id, og);
        return user ? { user, og } : null;
    },
    head: ({ loaderData, match, params }) => {
        const t = metaT(match.context.i18n);
        const locale = match.context.i18n?.locale;
        if (!loaderData?.og) return seo({ title: t("user.fallbackTitle"), path: `/user/${params.id}`, locale });
        const { user: profile, og } = loaderData;
        return seo({
            title: profile.nickname ?? "Player",
            // The profile's own resume wins; the fallback is one message with
            // both optional fragments selected inside it, so a translation can
            // reorder or drop the separators rather than inherit three
            // concatenated English pieces.
            description:
                profile.resume ||
                t("user.description", {
                    hasLevel: profile.level != null ? "yes" : "no",
                    hasGrade: profile.grade ? "yes" : "no",
                    level: profile.level,
                    grade: profile.grade,
                }),
            image: ogURL("user", params.id, og),
            path: `/user/${params.id}`,
            type: "profile",
            locale,
        });
    },
});

function RouteComponent() {
    // Keyed by uid: going from one profile to another remounts the page, so a
    // tab picked, a layout or showcase being edited, or a background draft on
    // the first never carries over to the second.
    const { id } = Route.useParams();
    return <UserProfile key={id} />;
}
