"use client";

import type { Operator } from "~/types/api";
import { ImageIcon } from "lucide-react";

interface ChibiViewerProps {
    operator: Operator;
}

export function ChibiViewer({ operator }: ChibiViewerProps) {
    return (
        <div className="rounded-xl border bg-card/50 p-6 backdrop-blur-sm">
            <h3 className="mb-4 text-lg font-semibold">Chibi Viewer</h3>
            <div className="flex aspect-video items-center justify-center rounded-lg border-2 border-dashed border-muted bg-muted/30">
                <div className="text-center">
                    <ImageIcon className="mx-auto mb-3 h-12 w-12 text-muted-foreground/50" />
                    <p className="text-sm font-medium text-muted-foreground">Chibi Viewer Coming Soon</p>
                    <p className="mt-1 text-xs text-muted-foreground/70">Spine animation viewer for {operator.name}</p>
                </div>
            </div>
        </div>
    );
}
