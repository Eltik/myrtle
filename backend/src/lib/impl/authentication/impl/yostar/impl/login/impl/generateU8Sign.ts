import crypto from "crypto";

export const generateU8Sign = (data: Record<string, any>): string => {
    // Sort the keys and create a URL query string
    const sortedEntries = Object.entries(data).sort(([keyA], [keyB]) => keyA.localeCompare(keyB));

    // Create URLSearchParams from entries
    const params = new URLSearchParams();
    sortedEntries.forEach(([key, value]) => {
        params.append(key, String(value));
    });
    const query = params.toString();

    // Create HMAC-SHA1 hash with the key
    const hmac = crypto.createHmac("sha1", "91240f70c09a08a6bc72af1a5c8d4670");
    hmac.update(query);

    // Return lowercase hex digest
    return hmac.digest("hex").toLowerCase();
};
