declare module "gif-encoder-2" {
    interface GIFEncoderOutput {
        getData(): Uint8Array;
    }

    class GIFEncoder {
        out: GIFEncoderOutput;

        constructor(width: number, height: number, algorithm?: "neuquant" | "octree", useOptimizer?: boolean, totalFrames?: number);

        setDelay(ms: number): void;
        setFramesPerSecond(fps: number): void;
        setQuality(quality: number): void;
        setThreshold(threshold: number): void;
        setRepeat(count: number): void;
        start(): void;
        addFrame(ctx: CanvasRenderingContext2D): void;
        finish(): void;

        on(event: "progress", callback: (percent: number) => void): void;
    }

    export default GIFEncoder;
}
