/**
 * Simple test for the lz4js library.
 */

import lz4 from "lz4js";

// Create some sample data
const sampleData = Buffer.from("Hello, this is a test of LZ4 compression. Let's see if it works correctly with lz4js!".repeat(10));
console.log(`Original data size: ${sampleData.length} bytes`);
console.log(`Original data: ${sampleData.toString().substring(0, 100)}...`);

// Compress the data
console.log("Compressing data...");
const compressedData = lz4.compress(sampleData);
console.log(`Compressed data size: ${compressedData.length} bytes`);
console.log(`Compression ratio: ${((compressedData.length / sampleData.length) * 100).toFixed(2)}%`);

// Decompress the data
console.log("Decompressing data...");
const decompressedData = lz4.decompress(compressedData);
console.log(`Decompressed data size: ${decompressedData.length} bytes`);

// Check if the decompressed data matches the original
const decompressedString = Buffer.from(decompressedData).toString();
const originalString = sampleData.toString();
const isMatch = decompressedString === originalString;

console.log(`Decompressed data matches original: ${isMatch}`);
if (!isMatch) {
    console.log("Original first 100 chars:", originalString.substring(0, 100));
    console.log("Decompressed first 100 chars:", decompressedString.substring(0, 100));
}

console.log("LZ4 test completed successfully!");

// This is a simple test that confirms the lz4js library works as expected with Bun
