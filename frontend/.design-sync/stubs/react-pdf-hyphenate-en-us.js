// @react-pdf/hyphenate@0.1.0 lib/en-us.js, re-pointed at the escaped copy of its
// index (stubs/react-pdf-hyphenate.js) through the bare specifier the paths map
// redirects. textkit imports this subpath; the subpath's relative `./index.js`
// would otherwise pull the raw, non-ASCII original back in.
import patterns from "hyphen/patterns/en-us.js";
import createHyphenator from "@react-pdf/hyphenate";

const { hyphenate, syllables } = createHyphenator(patterns);

export { hyphenate, syllables, patterns };
