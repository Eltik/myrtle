"use client";

import { ShieldAlert } from "lucide-react";
import { MotionDiv } from "./motion-primitives";
import { Card, CardContent, CardHeader, CardTitle } from "./shadcn/card";

export function AuthRequiredWall() {
    return (
        <div className="flex min-h-[60vh] items-center justify-center">
            <MotionDiv animate={{ opacity: 1, y: 0 }} initial={{ opacity: 0, y: 20 }} transition={{ duration: 0.5 }}>
                <Card className="w-full max-w-md">
                    <CardHeader className="flex flex-row items-center gap-4">
                        <ShieldAlert className="h-12 w-12 text-destructive" />
                        <CardTitle className="text-2xl">Authentication Required</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <p className="text-muted-foreground">You must be logged in to view this page. Please log in to continue.</p>
                    </CardContent>
                </Card>
            </MotionDiv>
        </div>
    );
}
