import { ReleaseError } from "frontend";

// A tab's load failure: the error message in destructive text and a retry button.
export const ServerError = () => (
    <div className="w-full max-w-xl p-4">
        <ReleaseError error={new Error("Couldn't reach the server. Check your connection and try again.")} onRetry={() => {}} />
    </div>
);

// A non-Error rejection with no message falls back to the generic copy.
export const GenericFallback = () => (
    <div className="w-full max-w-xl p-4">
        <ReleaseError error="" onRetry={() => {}} />
    </div>
);
