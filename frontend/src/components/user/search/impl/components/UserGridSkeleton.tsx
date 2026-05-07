import { PAGE_SIZE } from "../constants";
import { UserCardSkeleton } from "./UserCardSkeleton";

export function UserGridSkeleton() {
    return (
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {Array.from({ length: PAGE_SIZE }, (_, i) => `skel-${i}`).map((id) => (
                <UserCardSkeleton key={id} />
            ))}
        </div>
    );
}
