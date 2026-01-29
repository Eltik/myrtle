"use client";

import { AlertCircle } from "lucide-react";

interface ComponentFallbackProps {
    title?: string;
    message?: string;
    className?: string;
}

/**
 * Lightweight fallback UI for components that fail to load.
 * Used with ErrorBoundary for graceful degradation.
 */
export function ComponentFallback({ title = "Content unavailable", message = "This content could not be loaded", className = "" }: ComponentFallbackProps) {
    return (
        <div className={`flex flex-col items-center justify-center rounded-lg border border-border bg-muted/30 p-6 text-center ${className}`}>
            <AlertCircle className="mb-2 h-6 w-6 text-muted-foreground" />
            <h4 className="font-medium text-foreground text-sm">{title}</h4>
            <p className="mt-1 text-muted-foreground text-xs">{message}</p>
        </div>
    );
}
