import { Translations } from "frontend";

// Admin > Translations: locale picker, namespace filter, progress and the
// key/translation editor, with the Locales section under it. The design bundle
// has no signed-in user, so every query stays disabled and the screen holds its
// loading shell: header, four progress-card skeletons, the editor toolbar and
// row skeletons.
export const Loading = () => (
    <div className="w-full p-6">
        <Translations />
    </div>
);
