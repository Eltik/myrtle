import { Button, GithubIcon } from "frontend";
import { Heart } from "lucide-react";

export const HeaderActions = () => (
    <div className="flex w-full max-w-md items-center justify-between gap-4 rounded-lg border px-3 py-2">
        <div className="flex items-center gap-4">
            <span className="font-semibold text-sm">myrtle.moe</span>
            <span className="text-muted-foreground text-sm">Operators</span>
            <span className="text-muted-foreground text-sm">Stages</span>
            <span className="text-muted-foreground text-sm">Tools</span>
        </div>
        <div className="flex items-center gap-1">
            <Button aria-label="Support myrtle.moe" size="icon" variant="ghost">
                <Heart className="h-4 w-4" aria-hidden="true" />
            </Button>
            <Button aria-label="GitHub" size="icon" variant="ghost">
                <GithubIcon className="h-4 w-4" />
            </Button>
        </div>
    </div>
);

export const InButtons = () => (
    <div className="flex flex-wrap items-center gap-2">
        <Button variant="outline">
            <GithubIcon className="size-4" strokeWidth={1.9} />
            View source
        </Button>
        <Button variant="secondary">
            <GithubIcon className="size-4" strokeWidth={1.9} />
            Star on GitHub
        </Button>
        <Button aria-label="Open repository" size="icon" variant="outline">
            <GithubIcon className="size-4" strokeWidth={1.9} />
        </Button>
    </div>
);

export const Sizes = () => (
    <div className="flex items-center gap-6">
        <div className="flex flex-col items-center gap-2">
            <GithubIcon className="size-4" />
            <span className="text-muted-foreground text-xs">size-4</span>
        </div>
        <div className="flex flex-col items-center gap-2">
            <GithubIcon className="size-5" />
            <span className="text-muted-foreground text-xs">size-5</span>
        </div>
        <div className="flex flex-col items-center gap-2">
            <GithubIcon className="size-6" />
            <span className="text-muted-foreground text-xs">size-6</span>
        </div>
        <div className="flex flex-col items-center gap-2">
            <GithubIcon className="size-8" />
            <span className="text-muted-foreground text-xs">size-8</span>
        </div>
    </div>
);

export const StrokeWeights = () => (
    <div className="flex items-center gap-6">
        <div className="flex flex-col items-center gap-2">
            <GithubIcon className="size-8" strokeWidth={1.25} />
            <span className="text-muted-foreground text-xs">1.25</span>
        </div>
        <div className="flex flex-col items-center gap-2">
            <GithubIcon className="size-8" strokeWidth={1.9} />
            <span className="text-muted-foreground text-xs">1.9</span>
        </div>
        <div className="flex flex-col items-center gap-2">
            <GithubIcon className="size-8" strokeWidth={2.5} />
            <span className="text-muted-foreground text-xs">2.5</span>
        </div>
    </div>
);
