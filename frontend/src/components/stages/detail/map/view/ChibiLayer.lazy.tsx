import { ClientOnly } from "@tanstack/react-router";
import { lazy, Suspense } from "react";
import type { EnemyHoverFn } from "../impl/types";
import type { IChibiWalker } from "./ChibiLayer";

const LazyChibiLayer = lazy(() => import("./ChibiLayer").then((m) => ({ default: m.ChibiLayer })));

export function DynamicChibiLayer(props: { walkers: IChibiWalker[]; width: number; height: number; padY: number; tilt?: number; onEnemyHover?: EnemyHoverFn }) {
    return (
        <ClientOnly fallback={null}>
            <Suspense fallback={null}>
                <LazyChibiLayer {...props} />
            </Suspense>
        </ClientOnly>
    );
}
