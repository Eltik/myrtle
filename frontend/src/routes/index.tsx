import { createFileRoute } from "@tanstack/react-router";
import Home from "#/components/home/Home";
import { statsQueryOptions } from "#/lib/api/stats";

export const Route = createFileRoute("/")({
    component: Home,
    loader: ({ context }) => context.queryClient.ensureQueryData(statsQueryOptions()),
});
