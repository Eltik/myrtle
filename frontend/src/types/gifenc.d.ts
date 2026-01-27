declare module "gifenc" {
    interface GIFEncoderInstance {
        writeFrame(
            indexedPixels: Uint8Array,
            width: number,
            height: number,
            options?: {
                palette?: number[][];
                delay?: number;
                transparent?: boolean;
                transparentIndex?: number;
                dispose?: number;
            },
        ): void;
        finish(): void;
        bytes(): Uint8Array;
        bytesView(): Uint8Array;
    }

    export function GIFEncoder(): GIFEncoderInstance;
    export function quantize(rgba: Uint8Array | Uint8ClampedArray, maxColors: number, options?: { format?: string }): number[][];
    export function applyPalette(rgba: Uint8Array | Uint8ClampedArray, palette: number[][], format?: string): Uint8Array;
    export function nearestColorIndex(palette: number[][], color: number[]): number;
}
