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
    /** Which presentation surface this is; see the full note on `SceneIllust`'s own props.
     *  `"panel"` is the windowed operator-detail card, where the game does NOT play the
     *  entrance. ⚠️ This interface is a DUPLICATE of the one in `SceneIllust.tsx`, kept so the
     *  lazy wrapper does not pull the renderer into the SSR bundle; both must be edited together. */
    surface?: "viewer" | "panel";
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
