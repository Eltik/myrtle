declare module "gif.js" {
    interface GIFOptions {
        workers?: number;
        quality?: number;
        repeat?: number;
        background?: string;
        width?: number | null;
        height?: number | null;
        transparent?: number | null;
        dither?: boolean | string;
        debug?: boolean;
        workerScript?: string;
    }

    interface GIFFrameOptions {
        delay?: number;
        copy?: boolean;
        dispose?: number;
    }

    interface GIF {
        addFrame(image: CanvasRenderingContext2D | ImageData | HTMLCanvasElement | HTMLImageElement, options?: GIFFrameOptions): void;
        on(event: "start", callback: () => void): void;
        on(event: "progress", callback: (progress: number) => void): void;
        on(event: "finished", callback: (blob: Blob, data: Uint8Array) => void): void;
        on(event: "abort", callback: () => void): void;
        render(): void;
        abort(): void;
        running: boolean;
    }

    interface GIFConstructor {
        new (options?: GIFOptions): GIF;
    }

    const GIF: GIFConstructor;
    export default GIF;
}
