declare module 'lz4js' {
    /**
     * Decompresses data compressed with LZ4 algorithm
     * @param input The input compressed data as a Buffer or Uint8Array
     * @returns The decompressed data as a Uint8Array
     */
    export function decompress(input: Uint8Array | Buffer): Uint8Array;

    /**
     * Compresses data using LZ4 algorithm
     * @param input The input data as a Buffer or Uint8Array
     * @returns The compressed data as a Uint8Array
     */
    export function compress(input: Uint8Array | Buffer): Uint8Array;

    export default {
        decompress,
        compress
    };
} 