import { ReleaseLoading } from "frontend";

// Six skeleton rows in a card, shaped like the release lists: title and subtitle
// on the left, the date badge on the right.
export const Default = () => (
    <div className="w-full max-w-2xl p-4">
        <ReleaseLoading />
    </div>
);
