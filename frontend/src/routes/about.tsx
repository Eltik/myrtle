import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/about")({
    component: About,
});

function About() {
    return (
        <div className="p-8">
            <h1 className="text-4xl font-bold mb-4">About Us</h1>
            <p className="text-lg text-muted-foreground">
                This is the default about page for your TanStack Start application.
            </p>
        </div>
    );
}
