import { createFileRoute } from "@tanstack/react-router";
import { Archives } from "#/components/archives/Archives";
import { validateArchivesSearch } from "#/components/archives/Archives.constants";
import { archiveDemographicsQueryOptions, archiveDossiersQueryOptions, archiveNetworkQueryOptions, archiveOperatorQueryOptions } from "#/lib/api/archives";
import { resolveGamedataServer } from "#/lib/api/gamedata";
import { metaT } from "#/lib/meta";
import { defaultOgURL, ogURL } from "#/lib/og";
import { buildArchiveOgData } from "#/lib/og/impl/archive";
import { serverScopedOgId } from "#/lib/og/impl/ids";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/archives")({
    component: Archives,
    validateSearch: validateArchivesSearch,
    loaderDeps: ({ search }) => ({ tab: search.tab, id: search.id }),
    loader: async ({ context, deps }) => {
        const server = context.i18n.gamedataServer;
        const q = context.queryClient;
        // The open tab's data and an open file are prefetched; the rest load
        // when their tab opens. A failure here is left to the tab to report.
        const tab = deps.tab === "dossiers" ? q.ensureQueryData(archiveDossiersQueryOptions(server)) : deps.tab === "numbers" ? q.ensureQueryData(archiveDemographicsQueryOptions(server)) : deps.tab === "hidden" ? null : q.ensureQueryData(archiveNetworkQueryOptions(server));
        await Promise.all([tab?.catch(() => null), deps.id ? q.ensureQueryData(archiveOperatorQueryOptions(deps.id, server)).catch(() => null) : null]);
    },
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        // An open file unfurls as its dossier, from the record the loader prefetched;
        // no file, or one this server does not have, is the archives preset.
        const server = resolveGamedataServer(match.context.i18n.gamedataServer);
        const id = match.search.id;
        const file = id ? match.context.queryClient.getQueryData(archiveOperatorQueryOptions(id, server).queryKey) : null;
        const image = file ? ogURL("archive", serverScopedOgId(file.id, server), buildArchiveOgData(file, server)) : defaultOgURL("archives", match.context.i18n);
        const { meta, links } = seo({
            title: t("archives.title"),
            description: t("archives.description"),
            path: "/archives",
            image,
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});
