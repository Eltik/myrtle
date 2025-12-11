import Image from "next/image";
import { Dialog, DialogContent } from "~/components/ui/dialog";
import type { UISkin } from "~/types/impl/frontend/impl/operators";

interface FullscreenDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    imageSrc: string;
    operator: {
        name: string;
    };
    selectedSkinData: UISkin;
}

export function FullscreenDialog({ open, onOpenChange, imageSrc, operator, selectedSkinData }: FullscreenDialogProps) {
    return (
        <Dialog onOpenChange={onOpenChange} open={open}>
            <DialogContent className="flex h-[85vh] w-[95vw] max-w-6xl items-center justify-center bg-black/90 p-0 sm:h-[90vh]">
                <div className="relative flex h-full w-full items-center justify-center">
                    <Image alt={`${operator.name} - ${selectedSkinData.name} (Fullscreen)`} fill src={imageSrc} style={{ objectFit: "contain" }} unoptimized />
                </div>
            </DialogContent>
        </Dialog>
    );
}
