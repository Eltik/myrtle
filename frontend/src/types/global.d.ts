// Include spine player types
declare module "@esotericsoftware/spine-player" {
    export class SpinePlayer {
        constructor(parent: HTMLElement, config: SpinePlayerConfig);
        skeleton: {
            data: {
                animations: { name: string }[];
            };
        } | null;
        animationState: {
            timeScale: number;
        } | null;
        setAnimation(animationName: string): void;
        play(): void;
        pause(): void;
        dispose(): void;
    }

    export interface SpinePlayerConfig {
        jsonUrl?: string;
        skelUrl?: string;
        atlasUrl: string;
        animation?: string;
        alpha?: boolean;
        backgroundColor?: string;
        premultipliedAlpha?: boolean;
        showControls?: boolean;
        preserveDrawingBuffer?: boolean;
        viewport?: {
            debugRender?: boolean;
            x?: number;
            y?: number;
            width?: number;
            height?: number;
            padLeft?: number;
            padRight?: number;
            padTop?: number;
            padBottom?: number;
        };
        success?: (player: SpinePlayer) => void;
        error?: (error: unknown) => void;
    }
} 