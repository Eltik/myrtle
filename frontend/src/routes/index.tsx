import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/")({ component: Home });

function Home() {
    return (
        <div className="p-8">
            <h1 className="text-4xl font-bold mb-4">Welcome Home!</h1>
            <p className="text-lg text-muted-foreground">
                This is the default home page for your TanStack Start application.
            </p>
        </div>
    );
}
