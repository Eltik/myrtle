import { Toast } from "@base-ui/react/toast";
import { Button, Card, CardDescription, CardHeader, CardPanel, CardTitle, ToastProvider } from "frontend";
import { RefreshCwIcon, UploadIcon } from "lucide-react";
import { type ReactNode, useEffect, useState } from "react";

// The app mounts one ToastProvider in __root and pushes through the exported
// `toastManager` singleton. Previews get their own manager per story so the
// cards don't share a queue — `toastManager` is the only prop the provider
// forwards straight through to Base UI.

type Seed = {
    id: string;
    title: string;
    description?: string;
    type?: "success" | "error" | "info" | "warning" | "loading";
    actionProps?: { children: string };
};

type Position = "top-left" | "top-center" | "top-right" | "bottom-left" | "bottom-center" | "bottom-right";

function Stage({ position, seeds, children }: { position: Position; seeds: Seed[]; children: ReactNode }) {
    const [manager] = useState(() => Toast.createToastManager());

    useEffect(() => {
        for (const seed of seeds) {
            manager.add(seed);
        }
    }, [manager, seeds]);

    // timeout={0} keeps the toasts on screen; the app leaves the 5s default.
    return (
        <ToastProvider limit={5} position={position} timeout={0} toastManager={manager}>
            <div className="relative min-h-[520px] w-full">{children}</div>
        </ToastProvider>
    );
}

const SAVE_SEEDS: Seed[] = [
    { id: "plan-saved", title: "Plan saved", description: "6 operators, 31 materials queued for Chapter 8.", type: "success" },
    { id: "depot-synced", title: "Depot synced", description: "187 material counts pulled from the EN server.", type: "info" },
];

const ERROR_SEEDS: Seed[] = [{ id: "publish-failed", title: "Failed to publish", description: "/6-star-guards is already taken by another Doctor.", type: "error", actionProps: { children: "Retry" } }];

const TYPE_SEEDS: Seed[] = [
    { id: "t-warning", title: "Depot is out of date", description: "Last synced 9 days ago.", type: "warning" },
    { id: "t-info", title: "CN data imported", description: "3 new operators are now visible.", type: "info" },
    { id: "t-success", title: "Tier list published", description: "/6-star-guards is live.", type: "success" },
];

/** Default position — a success toast over the planner, with an older one stacked behind. */
export const BottomRight = () => (
    <Stage position="bottom-right" seeds={SAVE_SEEDS}>
        <Card className="max-w-md">
            <CardHeader>
                <CardTitle>Chapter 8 upgrade plan</CardTitle>
                <CardDescription>6 operators · 31 materials · 753 sanity remaining</CardDescription>
            </CardHeader>
            <CardPanel className="flex gap-2">
                <Button size="sm">
                    <UploadIcon />
                    Save plan
                </Button>
                <Button size="sm" variant="outline">
                    <RefreshCwIcon />
                    Sync depot
                </Button>
            </CardPanel>
        </Card>
    </Stage>
);

/** `position="top-center"` with an error toast and its retry action. */
export const TopCenterError = () => (
    <Stage position="top-center" seeds={ERROR_SEEDS}>
        <Card className="mt-16 max-w-md">
            <CardHeader>
                <CardTitle>Publish tier list</CardTitle>
                <CardDescription>6★ Guards by Chapter 8 clear speed</CardDescription>
            </CardHeader>
            <CardPanel className="flex gap-2">
                <Button size="sm">Publish</Button>
                <Button size="sm" variant="outline">
                    Save draft
                </Button>
            </CardPanel>
        </Card>
    </Stage>
);

/** Three queued toasts stack from the bottom-left; only the frontmost shows its content. */
export const StackedBottomLeft = () => (
    <Stage position="bottom-left" seeds={TYPE_SEEDS}>
        <Card className="ml-auto max-w-md">
            <CardHeader>
                <CardTitle>Depot</CardTitle>
                <CardDescription>187 material types tracked · last scanned 9 days ago</CardDescription>
            </CardHeader>
            <CardPanel className="flex gap-2">
                <Button size="sm" variant="outline">
                    Rescan
                </Button>
            </CardPanel>
        </Card>
    </Stage>
);
