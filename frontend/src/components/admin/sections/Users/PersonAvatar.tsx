import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { cn, DEFAULT_AVATAR_ID, getAvatarById } from "#/lib/utils";
import { initialsOf } from "./warnings";

/**
 * A player's in-game avatar, resolved the way the profile header, search and
 * leaderboard do (`avatar_id`, else Amiya). The initials show only while the
 * image loads or when it fails. Decorative: the name always sits beside it.
 */
export function PersonAvatar({ avatarId, name, className }: { avatarId: string | null; name: string; className?: string }): React.ReactElement {
    return (
        <Avatar className={cn("size-[30px] bg-muted", className)}>
            <AvatarImage src={getAvatarById(avatarId ?? DEFAULT_AVATAR_ID)} alt="" />
            <AvatarFallback className="font-semibold text-[11.5px]">{initialsOf(name)}</AvatarFallback>
        </Avatar>
    );
}
