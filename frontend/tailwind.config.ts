import { type Config } from "tailwindcss";
import { fontFamily } from "tailwindcss/defaultTheme";

export default {
    content: ["./src/**/*.tsx"],
    theme: {
        colors: ({ colors }) => ({
            inherit: colors.inherit,
            current: colors.current,
            transparent: colors.transparent,
            black: colors.black,
            white: colors.white,
            slate: colors.slate,
            gray: colors.gray,
            zinc: colors.zinc,
            neutral: colors.neutral,
            stone: colors.stone,
            red: colors.red,
            orange: colors.orange,
            amber: colors.amber,
            yellow: colors.yellow,
            lime: colors.lime,
            green: colors.green,
            emerald: colors.emerald,
            teal: colors.teal,
            cyan: colors.cyan,
            sky: colors.sky,
            blue: colors.blue,
            indigo: colors.indigo,
            violet: colors.violet,
            purple: colors.purple,
            fuchsia: colors.fuchsia,
            pink: colors.pink,
            rose: colors.rose,

            "main-pink-100": "rgb(107, 85, 89)", /** */
            "main-pink-200": "rgb(153, 121, 127)", /** */
            "main-pink-300": "rgb(194, 153, 161)", /** */
            "main-pink-400": "rgb(237, 187, 197)", /* navbar text, light pink of logo */
            "main-pink-500": "rgb(255, 201, 212)", /* titles that need color */

            "main-dark-pink-100": "rgb(135, 75, 53)", /** */
            "main-dark-pink-200": "rgb(189, 47, 80)", /** */
            "main-dark-pink-300": "rgb(220, 55, 94)", /* logo, navbar "myrtle.moe" */
            "main-dark-pink-400": "rgb(250, 92, 129)", /* highlights */

            "main-grey-100": "rgb(128, 128, 128)", /* description text */
            "main-grey-200": "rgb(171, 171, 171)", /* sub-subtitle, darker text */
            "main-grey-300": "rgb(220, 220, 220)", /* subtitles */

            "main-blue-100": "rgb(13, 14, 20)", /* background color */
            "main-blue-200": "rgb(24, 25, 37)", /* navigation bar background, boxes */
            "main-blue-300": "rgb(35, 36, 54)", /* boxes in boxes */
            "main-blue-400": "rgb(46, 48, 71)", /** */
            "main-blue-500": "rgb(68, 71, 105)", /** */
        }),
        extend: {
            fontFamily: {
                sans: ["var(--font-sans)", ...fontFamily.sans],
            }
        },
    },
    plugins: [require("@savvywombat/tailwindcss-grid-areas")],
} satisfies Config;
