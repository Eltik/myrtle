/** @type {import('prettier').Config & import('prettier-plugin-tailwindcss').PluginOptions} */
export default {
    plugins: ["prettier-plugin-tailwindcss"],
    tabWidth: 4,
    printWidth: 10000,
    proseWrap: "never",
    endOfLine: "lf",
    semi: true,
    singleQuote: false,
    bracketSpacing: true,
    bracketSameLine: false,
    arrowParens: "always",
    useTabs: false,
};
