import { ClientOnly } from "@tanstack/react-router";
import { lazy, Suspense } from "react";
import type { IChibiSpineFiles } from "#/lib/api/chibis";
import type { ISpineFit } from "../chibi/helpers";

interface ISceneIllustProps {
    files: IChibiSpineFiles;
    server?: "en" | "cn";
    fit?: ISpineFit;
    framing?: "character" | "authored";
    backdrop?: string;
    onReady?: () => void;
}

const LazySceneIllust = lazy(() => import("./SceneIllust").then((m) => ({ default: m.SceneIllust })));

/** SSR-safe wrapper: the PixiJS/Spine scene renderer only loads client-side. */
export function SceneIllustPlayer(props: ISceneIllustProps) {
    return (
        <ClientOnly fallback={null}>
            <Suspense fallback={null}>
                <LazySceneIllust {...props} />
            </Suspense>
        </ClientOnly>
    );
}
