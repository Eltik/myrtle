import { NewGridButton } from "frontend";

// "New grid": opens the create dialog for a signed-in user and the sign-in
// dialog for anyone else. The design bundle has no signed-in user, so the
// stories show the trigger as the browse hero and the My grids empty state place it.

export const Default = () => (
    <div className="p-4">
        <NewGridButton />
    </div>
);

/** Full width, as a phone's hero stacks it. */
export const FullWidth = () => (
    <div className="w-80 p-4">
        <NewGridButton className="w-full" />
    </div>
);
