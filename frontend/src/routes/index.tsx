import { createFileRoute } from "@tanstack/react-router";
import Home from "#/components/home/Home";
import { statsQueryOptions } from "#/lib/api/stats";
import { homeTierListsQueryOptions } from "#/lib/api/tier-lists";

export const Route = createFileRoute("/")({
    component: Home,
    loader: ({ context }) => Promise.all([context.queryClient.ensureQueryData(statsQueryOptions()), context.queryClient.ensureQueryData(homeTierListsQueryOptions())]),
});
